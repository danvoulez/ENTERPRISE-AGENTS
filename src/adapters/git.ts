import { execSync } from 'node:child_process';

export class GitAdapter {
  constructor(
    readonly repoRoot: string,
    private readonly remote: string,
    readonly branch: string,
    private readonly token?: string
  ) {}

  status(): string {
    return execSync('git status --short', { cwd: this.repoRoot, encoding: 'utf-8' });
  }

  add(files: string | string[]): void {
    const list = Array.isArray(files) ? files : [files];
    execSync(`git add ${list.map((file) => JSON.stringify(file)).join(' ')}`, { cwd: this.repoRoot });
  }

  commit(message: string): string {
    execSync(`git commit -m ${JSON.stringify(message)}`, { cwd: this.repoRoot });
    return execSync('git rev-parse HEAD', { cwd: this.repoRoot, encoding: 'utf-8' }).trim();
  }

  push(): void {
    if (this.token) {
      const currentUrl = execSync(`git remote get-url ${this.remote}`, { cwd: this.repoRoot, encoding: 'utf-8' }).trim();
      const tokenUrl = currentUrl.replace('https://', `https://${this.token}@`);
      execSync(`git remote set-url ${this.remote} ${JSON.stringify(tokenUrl)}`, { cwd: this.repoRoot });
      execSync(`git push ${this.remote} ${this.branch}`, { cwd: this.repoRoot });
      execSync(`git remote set-url ${this.remote} ${JSON.stringify(currentUrl)}`, { cwd: this.repoRoot });
      return;
    }

    execSync(`git push ${this.remote} ${this.branch}`, { cwd: this.repoRoot });
  }

  createBranch(name: string): void {
    execSync(`git checkout -b ${JSON.stringify(name)}`, { cwd: this.repoRoot });
  }

  checkout(branch: string): void {
    execSync(`git checkout ${JSON.stringify(branch)}`, { cwd: this.repoRoot });
  }

  diff(staged = false): string {
    return execSync(staged ? 'git diff --staged' : 'git diff', { cwd: this.repoRoot, encoding: 'utf-8' });
  }

  log(n: number): Array<{ sha: string; message: string; date: string }> {
    const output = execSync(`git log -n ${n} --pretty=format:%H%x09%s%x09%cI`, { cwd: this.repoRoot, encoding: 'utf-8' });
    return output
      .split('\n')
      .filter((line) => line.length > 0)
      .map((line) => {
        const [sha, message, date] = line.split('\t');
        return { sha, message, date };
      });
  }
}
