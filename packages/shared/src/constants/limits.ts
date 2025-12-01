export const MAX_ROOM_SIZE = 4;

export const RATE_LIMITS = {
  TIME_SYNC: { max: 10, window: 60000 }, // 10 per minute
  TRIGGER_CLIP: { max: 5, window: 60000 }, // 5 per minute
  JOIN_ROOM: { max: 10, window: 60000 }, // 10 per minute
  DEFAULT: { max: 20, window: 60000 }, // 20 per minute
} as const;
