import { FileWriterAdapter } from '../../adapters/file-writer.js';
import { ExecutionLogger } from '../../persistence/execution-logger.js';
import { ReviewOutput } from './review.js';

export const applyStage = async (
  fileWriter: FileWriterAdapter,
  reviewOutput: ReviewOutput,
  executionLogger: ExecutionLogger,
  jobId: string
): Promise<string[]> => {
  const modified: string[] = [];

  for (const file of reviewOutput.finalFiles) {
    let original: string | null = null;
    try {
      original = await fileWriter.readFile(file.path);
    } catch {
      original = null;
    }

    await fileWriter.writeFile(file.path, file.content);
    modified.push(file.path);

    executionLogger.logCodeChange({
      jobId,
      filePath: file.path,
      original,
      planned: original ?? '',
      reviewed: file.content,
      final: file.content,
      commitSha: ''
    });
  }

  return modified;
};
