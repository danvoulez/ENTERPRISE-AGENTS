export class EmailAdapter {
  constructor(
    private readonly apiUrl: string,
    private readonly apiKey: string,
    private readonly defaultTo: string
  ) {}

  async send(params: {
    to?: string;
    subject: string;
    body: string;
    html?: string;
  }): Promise<void> {
    const response = await fetch(this.apiUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${this.apiKey}`
      },
      body: JSON.stringify({
        to: params.to ?? this.defaultTo,
        subject: params.subject,
        text: params.body,
        html: params.html
      })
    });

    if (!response.ok) {
      throw new Error(`Email API failed with status ${response.status}`);
    }
  }

  async sendAlert(subject: string, body: string): Promise<void> {
    await this.send({ subject, body });
  }

  async sendMilestoneNotification(jobId: string, milestone: string): Promise<void> {
    await this.send({
      subject: `[Job ${jobId}] Milestone alcançado`,
      body: milestone
    });
  }

  async sendDecisionRequest(jobId: string, question: string, options: string[]): Promise<void> {
    await this.send({
      subject: `[Job ${jobId}] Decisão necessária`,
      body: `${question}\n\nOpções:\n- ${options.join('\n- ')}`
    });
  }
}
