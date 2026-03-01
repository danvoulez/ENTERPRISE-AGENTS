import Database from 'better-sqlite3';

export type JobStatus =
  | 'PENDING'
  | 'PLANNING'
  | 'CODING'
  | 'VALIDATING'
  | 'REVIEWING'
  | 'COMMITTING'
  | 'FAILED'
  | 'DONE';

export interface Job {
  id: string;
  issue_id: string;
  status: JobStatus;
  payload: string;
  retries: number;
}

export class JobsRepository {
  constructor(private readonly db: Database.Database) {}

  enqueue(job: Job): void {
    const now = new Date().toISOString();
    this.db
      .prepare(
        `INSERT OR REPLACE INTO jobs (id, issue_id, status, payload, retries, updated_at, created_at)
         VALUES (@id, @issue_id, @status, @payload, @retries, @updated_at, COALESCE((SELECT created_at FROM jobs WHERE id = @id), @created_at))`
      )
      .run({ ...job, updated_at: now, created_at: now });
  }

  nextPending(): Job | undefined {
    return this.db.prepare(`SELECT id, issue_id, status, payload, retries FROM jobs WHERE status = 'PENDING' ORDER BY created_at ASC LIMIT 1`).get() as Job | undefined;
  }

  updateStatus(id: string, status: JobStatus, lastError?: string): void {
    this.db
      .prepare('UPDATE jobs SET status = ?, last_error = ?, updated_at = ? WHERE id = ?')
      .run(status, lastError ?? null, new Date().toISOString(), id);
  }

  incrementRetries(id: string): void {
    this.db.prepare('UPDATE jobs SET retries = retries + 1, updated_at = ? WHERE id = ?').run(new Date().toISOString(), id);
  }
}
