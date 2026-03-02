use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use anyhow::{anyhow, bail, Result};

use crate::{
    adapters_rs::{AnthropicAdapter, GitAdapter, LinearAdapter, OllamaAdapter},
    branch_manager_rs::BranchManager,
    context_builder_rs::ContextBuilder,
    file_writer_rs::FileWriter,
    persistence_rs::{
        CheckpointStore, EvidenceStore, ExecutionLogger, Job, JobStatus, JobsRepository,
    },
    pr_creator_rs::PrCreator,
    state_machine_rs::StateMachine,
    test_runner_rs::TestRunner,
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
    branch_manager: BranchManager,
    file_writer: FileWriter,
    context_builder: ContextBuilder,
    test_runner: TestRunner,
    pr_creator: Option<PrCreator>,
    max_review_iterations: u8,
    linear_done_state_type: String,
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
        branch_manager: BranchManager,
        file_writer: FileWriter,
        context_builder: ContextBuilder,
        test_runner: TestRunner,
        pr_creator: Option<PrCreator>,
        max_review_iterations: u8,
        linear_done_state_type: String,
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
            branch_manager,
            file_writer,
            context_builder,
            test_runner,
            pr_creator,
            max_review_iterations,
            linear_done_state_type,
        }
    }

    pub async fn run(&self, mut job: Job) -> Result<()> {
        let issue = self.linear.get_issue(&job.issue_id).await?;
        let backlog = self.linear.list_team_issues(None).await?;
        let tracked = backlog.iter().any(|i| i.id == issue.id);
        if !tracked {
            return Err(anyhow!(
                "issue {} não pertence ao backlog do time configurado",
                issue.id
            ));
        }
        if issue.state.r#type.eq_ignore_ascii_case("completed") {
            self.transition(&mut job, JobStatus::Done)?;
            return Ok(());
        }

        let planning_prompt = self
            .context_builder
            .build_planning_prompt(&job.issue_id, &job.payload)
            .await?;

        self.branch_manager.ensure_clean().await?;
        let branch = self
            .branch_manager
            .create_job_branch(&issue.identifier)
            .await?;

        self.transition(&mut job, JobStatus::Planning)?;
        let plan = if let Some(saved) = self.checkpoint("PLANNING", &job.id) {
            saved
        } else {
            let generated = self
                .measure_and_log(&job.id, "plan", "anthropic", || {
                    self.anthropic.plan(&planning_prompt)
                })
                .await?;
            self.checkpoints
                .lock()
                .expect("checkpoint lock")
                .save(&job.id, "PLANNING", &generated);
            generated
        };

        self.transition(&mut job, JobStatus::Coding)?;
        let mut code = if let Some(saved) = self.checkpoint("CODING", &job.id) {
            saved
        } else {
            let generated = self
                .measure_and_log(&job.id, "code", "ollama", || self.ollama.code(&plan))
                .await?;
            self.checkpoints
                .lock()
                .expect("checkpoint lock")
                .save(&job.id, "CODING", &generated);
            generated
        };
        self.evidence.write(&job.id, "code", &code)?;

        self.transition(&mut job, JobStatus::Reviewing)?;
        let mut review = self
            .measure_and_log(&job.id, "review", "anthropic", || {
                self.anthropic.review(&code)
            })
            .await?;

        let mut iteration = 0;
        while !review.issues.is_empty() && iteration < self.max_review_iterations {
            code = self
                .measure_and_log(&job.id, "recoding", "ollama", || {
                    self.ollama.code(&review.code)
                })
                .await?;
            review = self
                .measure_and_log(&job.id, "rereview", "anthropic", || {
                    self.anthropic.review(&code)
                })
                .await?;
            iteration += 1;
        }

        self.evidence
            .write(&job.id, "review", &serde_json::to_string_pretty(&review)?)?;

        self.file_writer.write_from_llm_output(&code)?;

        self.transition(&mut job, JobStatus::Validating)?;
        let validation = self.test_runner.validate().await?;
        if !validation.passed {
            bail!("validação falhou: {}", validation.errors.join("; "));
        }

        let files = self.git.changed_files().await?;

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

        self.git.push_branch(&branch).await?;

        if let Some(pr_creator) = &self.pr_creator {
            let (number, url) = pr_creator
                .create(&job, &issue, &review, &branch, &files)
                .await?;
            self.evidence
                .write(&job.id, "pr", &format!("PR #{}: {}", number, url))?;
        }

        let done_state_id = self
            .linear
            .find_state_id_by_type(&self.linear_done_state_type)
            .await?;
        self.transition(&mut job, JobStatus::Done)?;
        self.linear
            .bulk_update_issue_state(&[job.issue_id.clone()], &done_state_id)
            .await?;
        Ok(())
    }

    fn checkpoint(&self, stage: &str, job_id: &str) -> Option<String> {
        self.checkpoints
            .lock()
            .expect("checkpoint lock")
            .get_latest(job_id, stage)
    }

    fn transition(&self, job: &mut Job, to: JobStatus) -> Result<()> {
        if job.status == to {
            return Ok(());
        }
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
