import { AnthropicAdapter } from '../adapters/anthropic.js';
import { EmailAdapter } from '../adapters/email.js';
import { SupabaseRealtimeAdapter } from '../adapters/supabase-realtime.js';
import { ExecutionLogger } from '../persistence/execution-logger.js';
import { JobsRepository } from '../persistence/jobs.js';

export class ConversationHandler {
  constructor(
    private readonly anthropic: AnthropicAdapter,
    private readonly supabase: SupabaseRealtimeAdapter,
    private readonly email: EmailAdapter,
    private readonly executionLogger: ExecutionLogger,
    private readonly jobs: JobsRepository
  ) {}

  async handleInbound(message: {
    source: 'webhook' | 'supabase';
    type: string;
    content: string;
    jobId?: string;
  }): Promise<void> {
    const context = message.jobId ? `job=${message.jobId}` : 'general';
    const response = await this.processWithLLM(message.content, context);

    this.executionLogger.logConversation({
      jobId: message.jobId,
      source: message.source,
      direction: 'inbound',
      messageType: message.type,
      payload: message
    });

    await this.supabase.send({
      type: 'response',
      content: response,
      jobId: message.jobId,
      timestamp: new Date().toISOString()
    });

    this.executionLogger.logConversation({
      jobId: message.jobId,
      source: 'system',
      direction: 'outbound',
      messageType: 'response',
      payload: { response }
    });
  }

  async alertOperator(params: {
    severity: 'info' | 'warning' | 'critical';
    message: string;
    jobId?: string;
    requiresResponse?: boolean;
  }): Promise<void> {
    await this.supabase.sendAlert(`[${params.severity}] ${params.message}`, params.jobId);
    await this.email.sendAlert(`[${params.severity}] ENTERPRISE-AGENTS`, params.message);
    if (params.requiresResponse && params.jobId) {
      await this.requestDecision({
        jobId: params.jobId,
        question: 'Responder ao alerta crítico?',
        options: ['sim', 'não']
      });
    }
  }

  async notifyMilestone(jobId: string, milestone: string, details: string): Promise<void> {
    await this.supabase.sendMilestone(`${milestone}: ${details}`, jobId);
    await this.email.sendMilestoneNotification(jobId, `${milestone}: ${details}`);
  }

  async requestDecision(params: {
    jobId: string;
    question: string;
    options: string[];
    deadline?: Date;
  }): Promise<void> {
    await this.supabase.sendDecisionRequest(params.question, params.jobId, params.options);
    await this.email.sendDecisionRequest(params.jobId, params.question, params.options);
  }

  private async processWithLLM(message: string, context: string): Promise<string> {
    const jobsState = this.jobs.nextPending();
    const prompt = `Context: ${context}\nPending: ${jobsState?.id ?? 'none'}\nMessage: ${message}`;
    return await this.anthropic.plan(prompt);
  }
}
