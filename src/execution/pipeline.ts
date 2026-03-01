import { AnthropicAdapter } from '../adapters/anthropic.js';
import { GitAdapter } from '../adapters/git.js';
import { OllamaAdapter } from '../adapters/ollama.js';
import { CheckpointStore } from '../control/checkpoint.js';
import { StateMachine } from '../control/state-machine.js';
import { AuditLog } from '../persistence/audit.js';
import { EvidenceStore } from '../persistence/evidence.js';
import { Job, JobsRepository, JobStatus } from '../persistence/jobs.js';
import { Metrics } from '../observability/metrics.js';
import { codeStage } from './stages/code.js';
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
    private readonly git: GitAdapter
  ) {}

  async run(job: Job): Promise<void> {
    await this.transition(job, 'PLANNING');
    const plan = await this.measure('plan', () => planStage(this.anthropic, job));
    this.checkpoints.save(job.id, 'PLANNING', { plan });

    await this.transition(job, 'CODING');
    const code = await this.measure('code', () => codeStage(this.ollama, plan));
    this.evidence.write(job.id, 'code', { code });

    await this.transition(job, 'VALIDATING');
    this.checkpoints.save(job.id, 'VALIDATING', { result: 'stubbed validation passed' });

    await this.transition(job, 'REVIEWING');
    const review = await this.measure('review', () => reviewStage(this.anthropic, code));
    this.evidence.write(job.id, 'review', { review });

    await this.transition(job, 'COMMITTING');
    this.audit.append(job.id, 'commit.skipped', { reason: 'pipeline demo mode', gitStatus: this.git.status() });

    await this.transition(job, 'DONE');
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

  private async measure<T>(stage: string, fn: () => Promise<T>): Promise<T> {
    const end = this.metrics.stageDuration.startTimer({ stage });
    try {
      return await fn();
    } catch (error) {
      this.metrics.stageFailures.inc({ stage });
      throw error;
    } finally {
      end();
    }
  }
}
