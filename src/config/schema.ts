import { z } from 'zod';

export const configSchema = z.object({
  anthropic: z.object({
    apiKey: z.string().startsWith('sk-ant-'),
    model: z.string().default('claude-sonnet-4-20250514'),
    maxTokens: z.number().default(4096),
    timeout: z.number().default(120_000),
    retries: z.number().default(3)
  }),
  ollama: z.object({
    baseUrl: z.string().url().default('http://localhost:11434'),
    model: z.string().default('qwen2.5-coder:7b'),
    maxTokens: z.number().default(4096),
    timeout: z.number().default(180_000),
    retries: z.number().default(2)
  }),
  linear: z.object({
    apiKey: z.string().startsWith('lin_api_'),
    teamKey: z.string(),
    project: z.string(),
    states: z.object({
      intake: z.array(z.string()).default(['Backlog', 'Todo']),
      working: z.string().default('In Progress'),
      done: z.string().default('Done'),
      failed: z.string().default('Backlog')
    })
  }),
  repo: z.object({
    root: z.string(),
    ciCommand: z.string().default('npm run ci:phase0'),
    branch: z.string().default('main')
  }),
  github: z.object({
    token: z.string(),
    remote: z.string().default('origin'),
    branch: z.string().default('main')
  }),
  supabase: z.object({
    url: z.string().url(),
    anonKey: z.string(),
    channel: z.string()
  }),
  webhook: z.object({
    port: z.number().default(9090),
    secret: z.string()
  }),
  email: z.object({
    apiUrl: z.string().url(),
    apiKey: z.string(),
    to: z.string().email()
  }),
  runtime: z.object({
    pollInterval: z.number().default(60_000),
    maxConcurrent: z.number().default(1),
    checkpointInterval: z.number().default(30_000)
  }),
  observability: z.object({
    logLevel: z.enum(['debug', 'info', 'warn', 'error']).default('info'),
    metricsPort: z.number().default(9090),
    healthPort: z.number().default(4001),
    dashboardPort: z.number().default(4000)
  }),
  alerts: z
    .object({
      slack: z.object({ webhookUrl: z.string().optional() }).optional(),
      email: z
        .object({
          smtp: z.string().optional(),
          from: z.string().optional(),
          to: z.string().optional()
        })
        .optional()
    })
    .optional(),
  db: z.object({
    path: z.string().default('./data/dual-agents.db'),
    auditPath: z.string().default('./data/audit.log'),
    evidencePath: z.string().default('./data/evidence/')
  })
});

export type DualAgentsConfig = z.infer<typeof configSchema>;
