import { execSync } from 'node:child_process';

export class GitAdapter {
  constructor(private readonly repoRoot: string) {}

  status(): string {
    return execSync('git status --short', { cwd: this.repoRoot, encoding: 'utf-8' });
  }

  commit(message: string): void {
    execSync('git add -A', { cwd: this.repoRoot });
    execSync(`git commit -m ${JSON.stringify(message)}`, { cwd: this.repoRoot });
  }
}
