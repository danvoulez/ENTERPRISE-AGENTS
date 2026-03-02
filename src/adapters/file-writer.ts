import { execSync } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';

export class FileWriterAdapter {
  constructor(private readonly repoRoot: string) {}

  async writeFile(relativePath: string, content: string): Promise<void> {
    const fullPath = path.join(this.repoRoot, relativePath);
    await mkdir(path.dirname(fullPath), { recursive: true });
    await writeFile(fullPath, content, 'utf-8');
  }

  async readFile(relativePath: string): Promise<string> {
    return await readFile(path.join(this.repoRoot, relativePath), 'utf-8');
  }

  async applyPatch(relativePath: string, patch: string): Promise<void> {
    const current = await this.readFile(relativePath);
    await this.writeFile(relativePath, `${current}\n${patch}`);
  }

  listModified(): string[] {
    const output = execSync('git status --short', { cwd: this.repoRoot, encoding: 'utf-8' });
    return output
      .split('\n')
      .map((line) => line.trim())
      .filter((line) => line.length > 0)
      .map((line) => line.slice(3).trim());
  }
}
