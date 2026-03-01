import { JobStatus } from '../persistence/jobs.js';

const transitions: Record<JobStatus, JobStatus[]> = {
  PENDING: ['PLANNING', 'FAILED'],
  PLANNING: ['CODING', 'FAILED'],
  CODING: ['VALIDATING', 'FAILED'],
  VALIDATING: ['REVIEWING', 'FAILED'],
  REVIEWING: ['COMMITTING', 'FAILED'],
  COMMITTING: ['DONE', 'FAILED'],
  FAILED: ['PENDING'],
  DONE: []
};

export class StateMachine {
  canTransition(from: JobStatus, to: JobStatus): boolean {
    return transitions[from].includes(to);
  }
}
