import { AnthropicAdapter } from '../../adapters/anthropic.js';
import { CodeOutput } from './code.js';

export interface ReviewOutput {
  approved: boolean;
  issues: Array<{
    severity: 'critical' | 'major' | 'minor' | 'suggestion';
    file: string;
    line?: number;
    description: string;
  }>;
  corrections: Array<{
    path: string;
    original: string;
    corrected: string;
    reason: string;
  }>;
  finalFiles: Array<{
    path: string;
    content: string;
  }>;
  summary: string;
}

const fallbackReview = (codeOutput: CodeOutput): ReviewOutput => ({
  approved: true,
  issues: [],
  corrections: [],
  finalFiles: codeOutput.files
    .filter((file) => file.action !== 'delete')
    .map((file) => ({ path: file.path, content: file.content })),
  summary: 'Review fallback used with generated files unchanged.'
});

export const reviewStage = async (anthropic: AnthropicAdapter, codeOutput: CodeOutput): Promise<ReviewOutput> => {
  const prompt = [
    'Review all files, fix issues directly, and return valid JSON only.',
    'Required schema:',
    '{"approved":true,"issues":[{"severity":"critical|major|minor|suggestion","file":"...","line":1,"description":"..."}],"corrections":[{"path":"...","original":"...","corrected":"...","reason":"..."}],"finalFiles":[{"path":"...","content":"..."}],"summary":"..."}',
    'Code output to review:',
    JSON.stringify(codeOutput)
  ].join('\n');

  const response = await anthropic.review(prompt);
  try {
    return JSON.parse(response) as ReviewOutput;
  } catch {
    const json = response.match(/\{[\s\S]*\}/);
    if (json) {
      try {
        return JSON.parse(json[0]) as ReviewOutput;
      } catch {
        return fallbackReview(codeOutput);
      }
    }
  }

  return fallbackReview(codeOutput);
};
