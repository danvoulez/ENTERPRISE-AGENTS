import { OllamaAdapter } from '../../adapters/ollama.js';

export interface CodeOutput {
  files: Array<{
    path: string;
    action: 'create' | 'modify' | 'delete';
    content: string;
  }>;
  summary: string;
}

const extractJson = (response: string): CodeOutput | null => {
  try {
    return JSON.parse(response) as CodeOutput;
  } catch {
    const match = response.match(/\{[\s\S]*\}/);
    if (!match) {
      return null;
    }
    try {
      return JSON.parse(match[0]) as CodeOutput;
    } catch {
      return null;
    }
  }
};

const extractFallback = (response: string): CodeOutput => {
  const blocks = [...response.matchAll(/```(?:[\w-]+)?\n([\s\S]*?)```/g)];
  const files = blocks.map((block, index) => ({
    path: `generated/file-${index + 1}.txt`,
    action: 'create' as const,
    content: block[1].trim()
  }));

  return {
    files,
    summary: 'Fallback parsing applied from code blocks.'
  };
};

export const codeStage = async (ollama: OllamaAdapter, plan: string, repoRoot: string): Promise<CodeOutput> => {
  const prompt = [
    `Repository root: ${repoRoot}`,
    'Return valid JSON only.',
    'Format: {"files":[{"path":"...","action":"create|modify|delete","content":"..."}],"summary":"..."}',
    'Each file must include path, action and full content.',
    'Task plan:',
    plan
  ].join('\n');

  const response = await ollama.code(prompt);
  const parsed = extractJson(response);
  if (parsed) {
    return parsed;
  }
  return extractFallback(response);
};
