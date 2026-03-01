import { Counter, Histogram, Registry, collectDefaultMetrics } from 'prom-client';

export class Metrics {
  readonly registry = new Registry();
  readonly stageDuration: Histogram<string>;
  readonly stageFailures: Counter<string>;
  readonly processedJobs: Counter<string>;

  constructor() {
    collectDefaultMetrics({ register: this.registry });
    this.stageDuration = new Histogram({
      name: 'dual_agents_stage_duration_seconds',
      help: 'Duration by stage',
      labelNames: ['stage'],
      registers: [this.registry]
    });
    this.stageFailures = new Counter({
      name: 'dual_agents_stage_failures_total',
      help: 'Failure count by stage',
      labelNames: ['stage'],
      registers: [this.registry]
    });
    this.processedJobs = new Counter({
      name: 'dual_agents_jobs_processed_total',
      help: 'Processed jobs counter',
      registers: [this.registry]
    });
  }
}
