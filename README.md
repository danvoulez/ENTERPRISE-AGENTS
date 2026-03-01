# Dual-Agents v1.0

Production-grade autonomous system with:

- Control plane (scheduler, state machine, SQLite queue/checkpoints)
- Execution plane (intake → plan → code → validate → review → commit)
- Integration adapters (Anthropic, Ollama, Linear, Git)
- Observability (structured logs, Prometheus metrics, SSE dashboard)
- Persistence (SQLite state + append-only audit + evidence store)

## Quick start

```bash
cp .env.example .env
npm install
npm run build
npm run start
```

Endpoints:

- Dashboard: `http://localhost:4000`
- Health: `http://localhost:4001/health`
- Metrics: `http://localhost:4001/metrics`
