import { AnthropicAdapter } from '../../adapters/anthropic.js';

export const reviewStage = async (anthropic: AnthropicAdapter, diff: string): Promise<string> => {
  return await anthropic.review(diff);
};
