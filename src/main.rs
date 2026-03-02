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
use tokio::time;
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

    let config = Config::from_env();
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
        AnthropicAdapter::new(config.anthropic_model.clone()),
        OllamaAdapter::new(config.ollama_model.clone()),
        GitAdapter::new(config.repo_root.clone(), config.git_branch.clone()),
        LinearAdapter,
    ));

    let poll_jobs = jobs.clone();
    let poll_pipeline = pipeline.clone();
    tokio::spawn(async move {
        let mut ticker = time::interval(Duration::from_secs(1));
        loop {
            ticker.tick().await;
            let maybe_job = poll_jobs.lock().expect("lock jobs").next_pending();
            if let Some(job) = maybe_job {
                if let Err(err) = poll_pipeline.run(job.clone()).await {
                    error!(job_id=%job.id, error=%err, "job failed");
                    let mut repo = poll_jobs.lock().expect("lock jobs");
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
    });

    api_rs::serve(config, jobs).await
}
