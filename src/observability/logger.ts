import pino from 'pino';

export const createLogger = (level: string) =>
  pino({ level, base: { service: 'dual-agents' }, timestamp: pino.stdTimeFunctions.isoTime });
