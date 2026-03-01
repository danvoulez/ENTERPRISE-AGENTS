export class OllamaAdapter {
  constructor(private readonly baseUrl: string, private readonly model: string) {}

  async code(task: string): Promise<string> {
    return `Generated code for ${task} with ${this.model} at ${this.baseUrl}`;
  }
}
