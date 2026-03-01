import { OllamaAdapter } from '../../adapters/ollama.js';

export const codeStage = async (ollama: OllamaAdapter, plan: string): Promise<string> => {
  return await ollama.code(plan);
};
