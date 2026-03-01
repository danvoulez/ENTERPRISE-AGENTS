export class AnthropicAdapter {
  constructor(private readonly apiKey: string, private readonly model: string) {}

  async plan(issue: string): Promise<string> {
    return `Plan for issue ${issue} using model ${this.model}`;
  }

  async review(diff: string): Promise<string> {
    return `Review for diff: ${diff.slice(0, 120)}`;
  }
}
