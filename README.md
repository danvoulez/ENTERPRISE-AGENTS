# Dual-Agents v1.1 (Rust-only)

Sistema fortalecido para produção e **100% em Rust**.

## O que está em produção

- Control plane com scheduler e máquina de estados
- Pipeline completo: plan → code → review → validate → commit
- API operacional com endpoints de health e listagem de jobs
- Persistência em SQLite (jobs, checkpoints e execution log)
- Evidências gravadas em filesystem
- Execução e deploy sem Node.js/TypeScript legado

## Executar localmente

```bash
cargo run
```

## Endpoints

- `GET /health` → `http://localhost:4001/health`
- `GET /jobs` → `http://localhost:4001/jobs`

## Configuração por ambiente

- `DB_PATH` (default: `dual_agents.db`)
- `EVIDENCE_PATH` (default: `evidence`)
- `REPO_ROOT` (default: `.`)
- `GIT_BRANCH` (default: `main`)
- `ANTHROPIC_MODEL` (default: `claude-3-5-sonnet`)
- `OLLAMA_MODEL` (default: `codellama`)
- `HEALTH_PORT` (default: `4001`)
- `POLL_INTERVAL_MS` (default: `1000`)

## Deploy

- Docker multi-stage build em `docker/Dockerfile`
- Service unit Rust em `systemd/dual-agents.service`
