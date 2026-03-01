import { LinearAdapter } from '../../adapters/linear.js';
import { Job } from '../../persistence/jobs.js';

export const intakeStage = async (linear: LinearAdapter): Promise<Job[]> => {
  return await linear.fetchIssues();
};
