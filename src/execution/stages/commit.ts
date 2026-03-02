import { GitAdapter } from '../../adapters/git.js';

export interface CommitOutput {
  sha: string;
  branch: string;
  filesCommitted: string[];
  message: string;
  pushed: boolean;
}

export const commitStage = async (
  git: GitAdapter,
  jobId: string,
  issueTitle: string,
  filesModified: string[],
  reviewSummary: string
): Promise<CommitOutput> => {
  git.add(filesModified);
  const message = `feat(${jobId}): ${issueTitle}\n\n${reviewSummary}`;
  const sha = git.commit(message);
  git.push();

  return {
    sha,
    branch: git.branch,
    filesCommitted: filesModified,
    message,
    pushed: true
  };
};
