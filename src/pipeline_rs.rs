use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use anyhow::{anyhow, Result};

use crate::{
    adapters_rs::{AnthropicAdapter, GitAdapter, LinearAdapter, OllamaAdapter},
    persistence_rs::{
        CheckpointStore, EvidenceStore, ExecutionLogger, Job, JobStatus, JobsRepository,
    },
    state_machine_rs::StateMachine,
};

pub struct Pipeline {
    jobs: Arc<Mutex<JobsRepository>>,
    checkpoints: Arc<Mutex<CheckpointStore>>,
    evidence: Arc<EvidenceStore>,
    execution_logger: Arc<Mutex<ExecutionLogger>>,
    fsm: StateMachine,
    anthropic: AnthropicAdapter,
    ollama: OllamaAdapter,
    git: GitAdapter,
    linear: LinearAdapter,
}

impl Pipeline {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        jobs: Arc<Mutex<JobsRepository>>,
        checkpoints: Arc<Mutex<CheckpointStore>>,
        evidence: Arc<EvidenceStore>,
        execution_logger: Arc<Mutex<ExecutionLogger>>,
        fsm: StateMachine,
        anthropic: AnthropicAdapter,
        ollama: OllamaAdapter,
        git: GitAdapter,
        linear: LinearAdapter,
    ) -> Self {
        Self {
            jobs,
            checkpoints,
            evidence,
            execution_logger,
            fsm,
            anthropic,
            ollama,
            git,
            linear,
        }
    }

    pub async fn run(&self, mut job: Job) -> Result<()> {
        self.transition(&mut job, JobStatus::Planning)?;
        let plan = self
            .measure_and_log(&job.id, "plan", "anthropic", || {
                self.anthropic.plan(&job.payload)
            })
            .await?;
        self.checkpoints
            .lock()
            .expect("checkpoint lock")
            .save(&job.id, "PLANNING", &plan);

        self.transition(&mut job, JobStatus::Coding)?;
        let code = self
            .measure_and_log(&job.id, "code", "ollama", || self.ollama.code(&plan))
            .await?;
        self.evidence.write(&job.id, "code", &code)?;

        self.transition(&mut job, JobStatus::Reviewing)?;
        let review = self
            .measure_and_log(&job.id, "review", "anthropic", || {
                self.anthropic.review(&code)
            })
            .await?;
        self.evidence
            .write(&job.id, "review", &serde_json::to_string_pretty(&review)?)?;

        self.transition(&mut job, JobStatus::Validating)?;
        // Validation/apply stage placeholder.
        let files = vec!["generated.patch".to_string()];

        self.transition(&mut job, JobStatus::Committing)?;
        let commit = self
            .git
            .commit(&job.id, "auto-commit", &files, &review.summary)
            .await?;
        self.execution_logger
            .lock()
            .expect("logger lock")
            .log_stage(
                &job.id,
                "commit",
                &serde_json::to_string(&files)?,
                &serde_json::to_string(&commit)?,
                "git",
                0,
            );

        self.transition(&mut job, JobStatus::Done)?;
        self.linear
            .update_issue_state(&job.issue_id, "Done")
            .await?;
        Ok(())
    }

    fn transition(&self, job: &mut Job, to: JobStatus) -> Result<()> {
        if !self.fsm.can_transition(job.status, to) {
            return Err(anyhow!("Invalid transition {:?} -> {:?}", job.status, to));
        }
        self.jobs
            .lock()
            .expect("jobs lock")
            .update_status(&job.id, to, None);
        job.status = to;
        Ok(())
    }

    async fn measure_and_log<T, F, Fut>(
        &self,
        job_id: &str,
        stage: &str,
        model: &str,
        f: F,
    ) -> Result<T>
    where
        T: serde::Serialize,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let start = Instant::now();
        let result = f().await?;
        let duration = start.elapsed().as_millis() as i64;
        self.execution_logger
            .lock()
            .expect("logger lock")
            .log_stage(
                job_id,
                stage,
                "(see checkpoints)",
                &serde_json::to_string(&result)?,
                model,
                duration,
            );
        Ok(result)
    }
}
