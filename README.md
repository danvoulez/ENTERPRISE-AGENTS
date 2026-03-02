# Dual-Agents v1.0

Production-grade autonomous system now running fully in **Rust** with:

- Control plane (scheduler, state machine, SQLite queue/checkpoints)
- Execution plane (plan → code → review → validate → commit)
- Integration adapters (Anthropic, Ollama, Linear, Git)
- Observability/API (Axum health/jobs endpoints + tracing logs)
- Persistence (SQLite state + execution logs + evidence store)

## Quick start

```bash
cargo run
```

Endpoints:

- Health: `http://localhost:4001/health`
- Jobs: `http://localhost:4001/jobs`
