import { View } from '../types/websocket';

export function now(): number {
  return Date.now();
}

/**
 * Calculates the Global Timeline Start (Epoch Ms)
 * This is the earliest start time among all synced clips.
 */
export function computeTimelineStartMs(views: View[]): number {
  if (views.length === 0) return 0;
  // Filter out invalid timestamps just in case
  const valid = views.filter((v) => v.videoStartTimeMs && v.videoStartTimeMs > 0);
  if (valid.length === 0) return 0;
  return Math.min(...valid.map((v) => v.videoStartTimeMs));
}

/**
 * Calculates the offset for a specific clip relative to the Global Timeline.
 * offset = clipStart - globalStart
 *
 * Example:
 * GlobalStart = 1000
 * ClipStart = 1500
 * Offset = 500 (Clip starts 500ms after the timeline begins)
 */
export function computeClipOffsetMs(view: View, timelineStartMs: number): number {
  if (!view.videoStartTimeMs) return 0;
  return Math.max(0, view.videoStartTimeMs - timelineStartMs);
}
