import { AnthropicAdapter } from '../adapters/anthropic.js';
import { GitAdapter } from '../adapters/git.js';
import { LinearAdapter } from '../adapters/linear.js';
import { OllamaAdapter } from '../adapters/ollama.js';
import { FileWriterAdapter } from '../adapters/file-writer.js';
import { CheckpointStore } from '../control/checkpoint.js';
import { ConversationHandler } from '../control/conversation-handler.js';
import { StateMachine } from '../control/state-machine.js';
import { AuditLog } from '../persistence/audit.js';
import { ExecutionLogger } from '../persistence/execution-logger.js';
import { EvidenceStore } from '../persistence/evidence.js';
import { Job, JobsRepository, JobStatus } from '../persistence/jobs.js';
import { Metrics } from '../observability/metrics.js';
import { applyStage } from './stages/apply.js';
import { codeStage } from './stages/code.js';
import { commitStage } from './stages/commit.js';
import { planStage } from './stages/plan.js';
import { reviewStage } from './stages/review.js';

export class Pipeline {
  constructor(
    private readonly jobs: JobsRepository,
    private readonly checkpoints: CheckpointStore,
    private readonly audit: AuditLog,
    private readonly evidence: EvidenceStore,
    private readonly metrics: Metrics,
    private readonly fsm: StateMachine,
    private readonly anthropic: AnthropicAdapter,
    private readonly ollama: OllamaAdapter,
    private readonly git: GitAdapter,
    private readonly fileWriter?: FileWriterAdapter,
    private readonly executionLogger?: ExecutionLogger,
    private readonly conversationHandler?: ConversationHandler,
    private readonly linear?: LinearAdapter
  ) {}

  async run(job: Job): Promise<void> {
    const startTime = Date.now();
    void startTime;

    await this.transition(job, 'PLANNING');
    const plan = await this.measureAndLog(job.id, 'plan', () => planStage(this.anthropic, job));
    this.checkpoints.save(job.id, 'PLANNING', { plan });

    await this.transition(job, 'CODING');
    const codeOutput = await this.measureAndLog(job.id, 'code', () => codeStage(this.ollama, plan, this.git.repoRoot));
    this.evidence.write(job.id, 'code', codeOutput);

    await this.transition(job, 'REVIEWING');
    const reviewOutput = await this.measureAndLog(job.id, 'review', () => reviewStage(this.anthropic, codeOutput));
    this.evidence.write(job.id, 'review', reviewOutput);

    if (reviewOutput.issues.some((issue) => issue.severity === 'critical')) {
      await this.conversationHandler!.alertOperator({
        severity: 'critical',
        message: `Job ${job.id} tem issues críticos na review`,
        jobId: job.id,
        requiresResponse: true
      });
    }

    await this.transition(job, 'VALIDATING');
    const modifiedFiles = await applyStage(this.fileWriter!, reviewOutput, this.executionLogger!, job.id);

    await this.transition(job, 'COMMITTING');
    const payload = JSON.parse(job.payload) as { title: string };
    const commitOutput = await commitStage(this.git, job.id, payload.title, modifiedFiles, reviewOutput.summary);

    this.executionLogger!.logStage({
      jobId: job.id,
      stage: 'commit',
      input: JSON.stringify({ files: modifiedFiles }),
      output: JSON.stringify(commitOutput),
      modelUsed: 'git',
      durationMs: 0
    });

    await this.transition(job, 'DONE');
    await this.linear!.updateIssueState(job.issue_id, 'Done');
    await this.conversationHandler!.notifyMilestone(
      job.id,
      'completed',
      `Commit ${commitOutput.sha} pushed to ${commitOutput.branch}`
    );

    this.metrics.processedJobs.inc();
  }

  private async transition(job: Job, to: JobStatus): Promise<void> {
    if (!this.fsm.canTransition(job.status, to)) {
      throw new Error(`Invalid transition ${job.status} -> ${to}`);
    }
    this.jobs.updateStatus(job.id, to);
    this.audit.append(job.id, 'state.transition', { from: job.status, to });
    job.status = to;
  }

  private async measureAndLog<T>(
    jobId: string,
    stage: 'plan' | 'code' | 'review' | 'commit',
    fn: () => Promise<T>
  ): Promise<T> {
    const end = this.metrics.stageDuration.startTimer({ stage });
    const start = Date.now();
    try {
      const result = await fn();
      const duration = Date.now() - start;
      this.executionLogger!.logStage({
        jobId,
        stage,
        input: '(see checkpoints)',
        output: typeof result === 'string' ? result : JSON.stringify(result),
        modelUsed: stage === 'code' ? 'ollama' : 'anthropic',
        durationMs: duration
      });
      return result;
    } catch (error) {
      this.metrics.stageFailures.inc({ stage });
      throw error;
    } finally {
      end();
    }
  }
}
