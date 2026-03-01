export class AlertDispatcher {
  async notify(message: string): Promise<void> {
    console.error(`[ALERT] ${message}`);
  }
}
