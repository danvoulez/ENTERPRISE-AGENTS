import { WorkQueue } from './queue.js';
import { Job } from '../persistence/jobs.js';

export class Scheduler {
  private timer?: NodeJS.Timeout;

  constructor(
    private readonly queue: WorkQueue,
    private readonly intervalMs: number,
    private readonly fetch: () => Promise<Job[]>
  ) {}

  start(): void {
    this.timer = setInterval(async () => {
      const jobs = await this.fetch();
      for (const job of jobs) {
        this.queue.push(job);
      }
    }, this.intervalMs);
  }

  stop(): void {
    if (this.timer) clearInterval(this.timer);
  }
}
