import { createClient, RealtimeChannel } from '@supabase/supabase-js';

export interface RealtimeMessage {
  type: 'alert' | 'command' | 'query' | 'response' | 'milestone' | 'decision';
  jobId?: string;
  content: string;
  metadata?: Record<string, unknown>;
  timestamp: string;
}

export class SupabaseRealtimeAdapter {
  private client: ReturnType<typeof createClient>;
  private channel: RealtimeChannel;
  private messageHandler?: (msg: RealtimeMessage) => void;

  constructor(
    private readonly url: string,
    private readonly anonKey: string,
    private readonly channelName: string
  ) {
    this.client = createClient(this.url, this.anonKey);
    this.channel = this.client.channel(this.channelName);
  }

  async connect(): Promise<void> {
    this.channel = this.client
      .channel(this.channelName)
      .on('broadcast', { event: 'message' }, (payload: { payload: unknown }) => {
        if (this.messageHandler) {
          this.messageHandler(payload.payload as RealtimeMessage);
        }
      })
      .subscribe();
  }

  onMessage(handler: (msg: RealtimeMessage) => void): void {
    this.messageHandler = handler;
  }

  async send(message: RealtimeMessage): Promise<void> {
    await this.channel.send({ type: 'broadcast', event: 'message', payload: message });
  }

  async sendAlert(content: string, jobId?: string): Promise<void> {
    await this.send({ type: 'alert', content, jobId, timestamp: new Date().toISOString() });
  }

  async sendMilestone(content: string, jobId: string): Promise<void> {
    await this.send({ type: 'milestone', content, jobId, timestamp: new Date().toISOString() });
  }

  async sendDecisionRequest(content: string, jobId: string, options?: string[]): Promise<void> {
    await this.send({
      type: 'decision',
      content,
      jobId,
      metadata: { options: options ?? [] },
      timestamp: new Date().toISOString()
    });
  }

  async disconnect(): Promise<void> {
    await this.channel.unsubscribe();
    await this.client.removeChannel(this.channel);
  }
}
