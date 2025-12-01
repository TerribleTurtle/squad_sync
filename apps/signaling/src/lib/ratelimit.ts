import { RATE_LIMITS } from '@squadsync/shared';

type ActionType = keyof typeof RATE_LIMITS;

interface RateLimitConfig {
  max: number;
  window: number;
}

export class RateLimiter {
  // Map<connectionId, Map<actionType, timestamp[]>>
  private requests: Map<string, Map<string, number[]>> = new Map();

  constructor() {
    // Periodically cleanup old entries to prevent memory leaks
    setInterval(() => this.cleanup(), 60000);
  }

  public checkLimit(connectionId: string, actionType: string): boolean {
    const now = Date.now();
    // Safe cast or fallback. We treat unknown types as DEFAULT.
    const config = RATE_LIMITS[actionType as ActionType] || RATE_LIMITS.DEFAULT;

    if (!this.requests.has(connectionId)) {
      this.requests.set(connectionId, new Map());
    }

    const userRequests = this.requests.get(connectionId)!;

    if (!userRequests.has(actionType)) {
      userRequests.set(actionType, []);
    }

    const timestamps = userRequests.get(actionType)!;

    // Filter out timestamps older than the window
    const validTimestamps = timestamps.filter((t) => now - t < config.window);

    // Update the list
    userRequests.set(actionType, validTimestamps);

    if (validTimestamps.length >= config.max) {
      return false; // Rate limited
    }

    // Add current request
    validTimestamps.push(now);
    return true; // Allowed
  }

  private cleanup() {
    const now = Date.now();
    for (const [connId, userRequests] of this.requests.entries()) {
      for (const [actionType, timestamps] of userRequests.entries()) {
        const config = RATE_LIMITS[actionType as ActionType] || RATE_LIMITS.DEFAULT;
        const validTimestamps = timestamps.filter((t) => now - t < config.window);

        if (validTimestamps.length === 0) {
          userRequests.delete(actionType);
        } else {
          userRequests.set(actionType, validTimestamps);
        }
      }

      if (userRequests.size === 0) {
        this.requests.delete(connId);
      }
    }
  }
}
