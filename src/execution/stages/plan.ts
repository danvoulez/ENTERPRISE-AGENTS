import { AnthropicAdapter } from '../../adapters/anthropic.js';
import { Job } from '../../persistence/jobs.js';

export const planStage = async (anthropic: AnthropicAdapter, job: Job): Promise<string> => {
  return await anthropic.plan(job.issue_id);
};
