import crypto from 'node:crypto';
import Database from 'better-sqlite3';

export class ExecutionLogger {
  constructor(private readonly db: Database.Database) {}

  logStage(params: {
    jobId: string;
    stage: 'plan' | 'code' | 'review' | 'commit';
    input: string;
    output: string;
    tokensUsed?: number;
    modelUsed: string;
    durationMs: number;
  }): number {
    const inputHash = crypto.createHash('sha256').update(params.input).digest('hex');
    const outputHash = crypto.createHash('sha256').update(params.output).digest('hex');
    const result = this.db
      .prepare(
        `INSERT INTO execution_log (job_id, stage, input_hash, output_text, output_hash, tokens_used, model_used, duration_ms)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)`
      )
      .run(params.jobId, params.stage, inputHash, params.output, outputHash, params.tokensUsed ?? null, params.modelUsed, params.durationMs);
    return Number(result.lastInsertRowid);
  }

  logCodeChange(params: {
    jobId: string;
    filePath: string;
    original: string | null;
    planned: string;
    reviewed: string;
    final: string;
    commitSha: string;
  }): number {
    const result = this.db
      .prepare(
        `INSERT INTO code_changes (job_id, file_path, original_content, planned_content, reviewed_content, final_content, commit_sha)
         VALUES (?, ?, ?, ?, ?, ?, ?)`
      )
      .run(params.jobId, params.filePath, params.original, params.planned, params.reviewed, params.final, params.commitSha);
    return Number(result.lastInsertRowid);
  }

  logConversation(params: {
    jobId?: string;
    source: 'webhook' | 'supabase' | 'system';
    direction: 'inbound' | 'outbound';
    messageType: string;
    payload: unknown;
  }): number {
    const result = this.db
      .prepare('INSERT INTO conversations (job_id, source, direction, message_type, payload) VALUES (?, ?, ?, ?, ?)')
      .run(params.jobId ?? null, params.source, params.direction, params.messageType, JSON.stringify(params.payload));
    return Number(result.lastInsertRowid);
  }

  getJobHistory(jobId: string): {
    stages: Array<{ stage: string; output: string; duration: number }>;
    changes: Array<{ file: string; commitSha: string }>;
    conversations: Array<{ direction: string; payload: string }>;
  } {
    const stages = this.db
      .prepare('SELECT stage, output_text as output, duration_ms as duration FROM execution_log WHERE job_id = ? ORDER BY id ASC')
      .all(jobId) as Array<{ stage: string; output: string; duration: number }>;
    const changes = this.db
      .prepare('SELECT file_path as file, commit_sha as commitSha FROM code_changes WHERE job_id = ? ORDER BY id ASC')
      .all(jobId) as Array<{ file: string; commitSha: string }>;
    const conversations = this.db
      .prepare('SELECT direction, payload FROM conversations WHERE job_id = ? ORDER BY id ASC')
      .all(jobId) as Array<{ direction: string; payload: string }>;

    return { stages, changes, conversations };
  }
}
