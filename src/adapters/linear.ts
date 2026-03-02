import { Job } from '../persistence/jobs.js';

export class LinearAdapter {
  constructor(private readonly teamKey: string, private readonly project: string) {}

  async updateIssueState(_issueId: string, _state: string): Promise<void> {
    return;
  }

  async fetchIssues(): Promise<Job[]> {
    return [
      {
        id: `${this.teamKey}-${Date.now()}`,
        issue_id: `${this.project}-seed`,
        status: 'PENDING',
        payload: JSON.stringify({ title: 'Seed issue from Linear adapter' }),
        retries: 0
      }
    ];
  }
}
