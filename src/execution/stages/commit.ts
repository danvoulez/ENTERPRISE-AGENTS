import { GitAdapter } from '../../adapters/git.js';

export const commitStage = (git: GitAdapter, message: string): void => {
  git.commit(message);
};
