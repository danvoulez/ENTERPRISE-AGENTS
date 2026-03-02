mod adapters_rs;
mod api_rs;
mod branch_manager_rs;
mod config_rs;
mod context_builder_rs;
mod file_writer_rs;
mod persistence_rs;
mod pipeline_rs;
mod pr_creator_rs;
mod state_machine_rs;
mod test_runner_rs;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Result;
use tokio::{signal, sync::Semaphore, task::JoinHandle, time};
use tracing::{error, info};

use adapters_rs::{AnthropicAdapter, GitAdapter, LinearAdapter, OllamaAdapter};
use branch_manager_rs::BranchManager;
use config_rs::Config;
use context_builder_rs::ContextBuilder;
use file_writer_rs::FileWriter;
use persistence_rs::{CheckpointStore, EvidenceStore, ExecutionLogger, JobsRepository, SqliteDb};
use pipeline_rs::Pipeline;
use pr_creator_rs::PrCreator;
use state_machine_rs::StateMachine;
use test_runner_rs::TestRunner;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::from_env()?;
    let db = SqliteDb::open(&config.db_path)?;
    db.run_migrations()?;

    let jobs = Arc::new(Mutex::new(JobsRepository::new(db.connection())));
    let checkpoints = Arc::new(Mutex::new(CheckpointStore::new(db.connection())));
    let evidence = Arc::new(EvidenceStore::new(config.evidence_path.clone()));
    let execution_logger = Arc::new(Mutex::new(ExecutionLogger::new(db.connection())));

    let linear = LinearAdapter::new(config.linear_api_key.clone(), config.linear_team_id.clone());
    let git = GitAdapter::new(
        config.repo_root.clone(),
        config.git_branch.clone(),
        config.git_remote.clone(),
    );

    let pr_creator = match (&config.github_token, &config.github_repo) {
        (Some(token), Some(repo)) => Some(PrCreator::new(
            token.clone(),
            repo.clone(),
            config.git_branch.clone(),
        )),
        _ => None,
    };

    let pipeline = Arc::new(Pipeline::new(
        jobs.clone(),
        checkpoints,
        evidence,
        execution_logger,
        StateMachine::default(),
        AnthropicAdapter::new(
            config.anthropic_model.clone(),
            config.anthropic_api_key.clone(),
        ),
        OllamaAdapter::new(config.ollama_model.clone(), config.ollama_base_url.clone()),
        git.clone(),
        linear.clone(),
        BranchManager::new(git.clone()),
        FileWriter::new(config.repo_root.clone()),
        ContextBuilder::new(config.voulezvous_spec_path.clone(), linear),
        TestRunner::new(config.repo_root.clone()),
        pr_creator,
        config.max_review_iterations,
        config.linear_done_state_type.clone(),
    ));

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let worker = spawn_worker(
        jobs.clone(),
        pipeline,
        config.poll_interval_ms,
        config.max_concurrent_jobs,
        shutdown_rx,
    );
    let api = tokio::spawn(api_rs::serve(config.clone(), jobs));

    signal::ctrl_c().await?;
    info!("shutdown signal received");
    let _ = shutdown_tx.send(true);

    worker.await??;
    api.abort();
    Ok(())
}

fn spawn_worker(
    jobs: Arc<Mutex<JobsRepository>>,
    pipeline: Arc<Pipeline>,
    poll_interval_ms: u64,
    max_concurrent_jobs: usize,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        let semaphore = Arc::new(Semaphore::new(max_concurrent_jobs));
        let mut ticker = time::interval(Duration::from_millis(poll_interval_ms));
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let maybe_job = jobs.lock().expect("lock jobs").next_pending();
                    if let Some(job) = maybe_job {
                        let permit = semaphore.clone().acquire_owned().await.expect("semaphore closed");
                        let jobs_ref = jobs.clone();
                        let pipeline_ref = pipeline.clone();
                        tokio::spawn(async move {
                            let _permit = permit;
                            if let Err(err) = pipeline_ref.run(job.clone()).await {
                                error!(job_id=%job.id, error=%err, "job failed");
                                let mut repo = jobs_ref.lock().expect("lock jobs");
                                repo.increment_retries(&job.id);
                                repo.update_status(
                                    &job.id,
                                    persistence_rs::JobStatus::Failed,
                                    Some(err.to_string()),
                                );
                            } else {
                                info!(job_id=%job.id, "job completed");
                            }
                        });
                    }
                }
                changed = shutdown.changed() => {
                    if changed.is_ok() && *shutdown.borrow() {
                        info!("worker shutdown complete");
                        return Ok(());
                    }
                }
            }
        }
    })
}
