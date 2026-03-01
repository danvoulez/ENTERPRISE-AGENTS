import express from 'express';
import { Metrics } from '../observability/metrics.js';

export const healthRouter = (metrics: Metrics) => {
  const router = express.Router();
  router.get('/health', (_req, res) => res.json({ status: 'ok' }));
  router.get('/ready', (_req, res) => res.json({ ready: true }));
  router.get('/metrics', async (_req, res) => {
    res.set('Content-Type', metrics.registry.contentType);
    res.end(await metrics.registry.metrics());
  });
  return router;
};
