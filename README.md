# Dual-Agents v1.2 (Rust-only)

Sistema fortalecido para produção e **100% em Rust**.

## O que está em produção

- Control plane com scheduler e máquina de estados
- Pipeline completo e idempotente: plan → code → review → validate → commit
- API operacional com endpoints de health e listagem de jobs
- Persistência em SQLite (jobs, checkpoints e execution log)
- Evidências gravadas em filesystem
- Adapters reais para Anthropic, Ollama, Git e Linear (GraphQL)
- Commit + push automático para o remote/branch configurado no ambiente

## Executar localmente

```bash
cargo run
```

## Endpoints

- `GET /health` → `http://localhost:4001/health`
- `GET /jobs` → `http://localhost:4001/jobs`

## Configuração por ambiente

Todas as configurações foram centralizadas no arquivo `.env`.

- `DB_PATH` (default: `dual_agents.db`)
- `EVIDENCE_PATH` (default: `evidence`)
- `REPO_ROOT` (default: `.`)
- `GIT_BRANCH` (default: `main`)
- `GIT_REMOTE` (default: `origin`)
- `ANTHROPIC_MODEL` (default: `claude-3-5-sonnet-20241022`)
- `ANTHROPIC_API_KEY` (opcional; sem chave usa fallback local no planning/review)
- `OLLAMA_MODEL` (default: `codellama`)
- `OLLAMA_BASE_URL` (default: `http://localhost:11434`)
- `LINEAR_API_KEY` (**obrigatório**)
- `LINEAR_TEAM_ID` (**obrigatório**)
- `LINEAR_DONE_STATE_TYPE` (default: `completed`)
- `HEALTH_PORT` (default: `4001`)
- `POLL_INTERVAL_MS` (default: `1000`)
- `MAX_REVIEW_ITERATIONS` (default: `2`)

## Deploy

- Docker multi-stage build em `docker/Dockerfile`
- Service unit Rust em `systemd/dual-agents.service`
