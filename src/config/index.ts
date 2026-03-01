import dotenv from 'dotenv';
import { configSchema, DualAgentsConfig } from './schema.js';
import { secret } from './secrets.js';

dotenv.config();

export const loadConfig = (): DualAgentsConfig => {
  const parsed = configSchema.parse({
    anthropic: {
      apiKey: secret(process.env.ANTHROPIC_API_KEY, 'sk-ant-placeholder')
    },
    ollama: {
      baseUrl: process.env.OLLAMA_BASE_URL
    },
    linear: {
      apiKey: secret(process.env.LINEAR_API_KEY, 'lin_api_placeholder'),
      teamKey: secret(process.env.LINEAR_TEAM_KEY, 'TEAM'),
      project: secret(process.env.LINEAR_PROJECT, 'Project'),
      states: {}
    },
    repo: {
      root: process.env.REPO_ROOT ?? process.cwd(),
      ciCommand: process.env.REPO_CI_COMMAND,
      branch: process.env.REPO_BRANCH
    },
    runtime: {
      pollInterval: Number(process.env.RUNTIME_POLL_INTERVAL ?? 60000),
      maxConcurrent: Number(process.env.RUNTIME_MAX_CONCURRENT ?? 1),
      checkpointInterval: Number(process.env.RUNTIME_CHECKPOINT_INTERVAL ?? 30000)
    },
    observability: {
      logLevel: (process.env.LOG_LEVEL as 'debug' | 'info' | 'warn' | 'error' | undefined) ?? 'info',
      metricsPort: Number(process.env.METRICS_PORT ?? 9090),
      healthPort: Number(process.env.HEALTH_PORT ?? 4001),
      dashboardPort: Number(process.env.DASHBOARD_PORT ?? 4000)
    },
    db: {
      path: process.env.DB_PATH,
      auditPath: process.env.AUDIT_PATH,
      evidencePath: process.env.EVIDENCE_PATH
    }
  });

  return parsed;
};
