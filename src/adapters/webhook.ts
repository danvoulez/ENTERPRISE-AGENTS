import crypto from 'node:crypto';
import express, { Router } from 'express';
import { ExecutionLogger } from '../persistence/execution-logger.js';

export interface WebhookPayload {
  type: string;
  source: string;
  jobId?: string;
  message: string;
  data?: Record<string, unknown>;
}

export class WebhookAdapter {
  private handlers: Map<string, (payload: WebhookPayload) => Promise<void>> = new Map();

  constructor(
    private readonly secret: string,
    private readonly executionLogger: ExecutionLogger
  ) {}

  getRouter(): Router {
    const router = express.Router();

    router.post('/webhook', async (req, res) => {
      const payloadText = JSON.stringify(req.body);
      const signature = req.header('X-Webhook-Signature') ?? '';

      if (!this.validateSignature(payloadText, signature)) {
        res.status(401).json({ error: 'invalid signature' });
        return;
      }

      const payload = req.body as WebhookPayload;
      this.executionLogger.logConversation({
        jobId: payload.jobId,
        source: 'webhook',
        direction: 'inbound',
        messageType: payload.type,
        payload
      });

      const handler = this.handlers.get(payload.type);
      if (handler) {
        await handler(payload);
      }

      res.status(202).json({ ok: true });
    });

    return router;
  }

  on(type: string, handler: (payload: WebhookPayload) => Promise<void>): void {
    this.handlers.set(type, handler);
  }

  private validateSignature(payload: string, signature: string): boolean {
    const digest = crypto.createHmac('sha256', this.secret).update(payload).digest('hex');
    return signature === digest;
  }
}
