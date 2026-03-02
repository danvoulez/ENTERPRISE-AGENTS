mod adapters_rs;
mod api_rs;
mod config_rs;
mod persistence_rs;
mod pipeline_rs;
mod state_machine_rs;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Result;
use tokio::{signal, task::JoinHandle, time};
use tracing::{error, info};

use adapters_rs::{AnthropicAdapter, GitAdapter, LinearAdapter, OllamaAdapter};
use config_rs::Config;
use persistence_rs::{CheckpointStore, EvidenceStore, ExecutionLogger, JobsRepository, SqliteDb};
use pipeline_rs::Pipeline;
use state_machine_rs::StateMachine;

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
        GitAdapter::new(
            config.repo_root.clone(),
            config.git_branch.clone(),
            config.git_remote.clone(),
        ),
        LinearAdapter::new(config.linear_api_key.clone(), config.linear_team_id.clone()),
        config.max_review_iterations,
        config.linear_done_state_type.clone(),
    ));

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let worker = spawn_worker(jobs.clone(), pipeline, config.poll_interval_ms, shutdown_rx);
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
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> JoinHandle<Result<()>> {
    tokio::spawn(async move {
        let mut ticker = time::interval(Duration::from_millis(poll_interval_ms));
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let maybe_job = jobs.lock().expect("lock jobs").next_pending();
                    if let Some(job) = maybe_job {
                        if let Err(err) = pipeline.run(job.clone()).await {
                            error!(job_id=%job.id, error=%err, "job failed");
                            let mut repo = jobs.lock().expect("lock jobs");
                            repo.increment_retries(&job.id);
                            repo.update_status(
                                &job.id,
                                persistence_rs::JobStatus::Failed,
                                Some(err.to_string()),
                            );
                        } else {
                            info!(job_id=%job.id, "job completed");
                        }
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
