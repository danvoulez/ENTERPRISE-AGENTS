import Database from 'better-sqlite3';

export class CheckpointStore {
  constructor(private readonly db: Database.Database) {}

  save(jobId: string, stage: string, data: unknown): void {
    this.db
      .prepare('INSERT INTO checkpoints (job_id, stage, data, created_at) VALUES (?, ?, ?, ?)')
      .run(jobId, stage, JSON.stringify(data), new Date().toISOString());
  }
}
