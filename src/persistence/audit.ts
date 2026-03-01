import fs from 'node:fs';
import path from 'node:path';

export class AuditLog {
  constructor(private readonly filePath: string) {
    fs.mkdirSync(path.dirname(this.filePath), { recursive: true });
  }

  append(jobId: string, eventType: string, payload: unknown): void {
    const record = {
      ts: new Date().toISOString(),
      jobId,
      eventType,
      payload
    };
    fs.appendFileSync(this.filePath, `${JSON.stringify(record)}\n`);
  }
}
