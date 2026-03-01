import fs from 'node:fs';
import path from 'node:path';

export class EvidenceStore {
  constructor(private readonly root: string) {
    fs.mkdirSync(this.root, { recursive: true });
  }

  write(jobId: string, name: string, payload: unknown): string {
    const dir = path.join(this.root, jobId);
    fs.mkdirSync(dir, { recursive: true });
    const file = path.join(dir, `${Date.now()}-${name}.json`);
    fs.writeFileSync(file, JSON.stringify(payload, null, 2));
    return file;
  }
}
