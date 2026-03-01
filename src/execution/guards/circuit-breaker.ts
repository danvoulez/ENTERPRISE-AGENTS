export class CircuitBreaker {
  private failures = 0;
  private openUntil = 0;

  constructor(private readonly threshold = 3, private readonly cooldownMs = 60_000) {}

  async execute<T>(fn: () => Promise<T>): Promise<T> {
    if (Date.now() < this.openUntil) {
      throw new Error('Circuit breaker is open');
    }
    try {
      const value = await fn();
      this.failures = 0;
      return value;
    } catch (error) {
      this.failures += 1;
      if (this.failures >= this.threshold) {
        this.openUntil = Date.now() + this.cooldownMs;
      }
      throw error;
    }
  }
}
