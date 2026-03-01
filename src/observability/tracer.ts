import crypto from 'node:crypto';

export interface Span {
  id: string;
  parentId?: string;
  name: string;
  startedAt: number;
}

export class Tracer {
  start(name: string, parentId?: string): Span {
    return { id: crypto.randomUUID(), name, parentId, startedAt: Date.now() };
  }

  end(span: Span): number {
    return Date.now() - span.startedAt;
  }
}
