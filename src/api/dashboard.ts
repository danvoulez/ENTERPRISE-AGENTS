import express from 'express';
import path from 'node:path';

export const dashboardRouter = () => {
  const router = express.Router();
  router.get('/events', (_req, res) => {
    res.setHeader('Content-Type', 'text/event-stream');
    res.setHeader('Cache-Control', 'no-cache');
    res.write(`data: ${JSON.stringify({ ts: Date.now(), status: 'alive' })}\n\n`);
    const timer = setInterval(() => {
      res.write(`data: ${JSON.stringify({ ts: Date.now(), status: 'tick' })}\n\n`);
    }, 5000);
    res.on('close', () => clearInterval(timer));
  });

  router.get('/', (_req, res) => {
    res.sendFile(path.resolve('dashboard/index.html'));
  });

  return router;
};
