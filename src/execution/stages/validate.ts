import { execSync } from 'node:child_process';

export const validateStage = (command: string, cwd: string): string => {
  return execSync(command, { cwd, encoding: 'utf-8' });
};
