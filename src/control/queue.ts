import { JobsRepository, type Job } from '../persistence/jobs.js';

export class WorkQueue {
  constructor(private readonly jobs: JobsRepository) {}

  push(job: Job): void {
    this.jobs.enqueue(job);
  }

  pull(): Job | undefined {
    return this.jobs.nextPending();
  }
}
