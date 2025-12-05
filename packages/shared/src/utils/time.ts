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
  // Use Math.max to find the LATEST start time (Intersection of all clips)
  // This ensures the timeline starts only when ALL clips have started.
  return Math.max(...valid.map((v) => v.videoStartTimeMs));
}

/**
 * Calculates the Global Timeline End (Epoch Ms)
 * This is the earliest end time among all synced clips (Intersection).
 */
export function computeTimelineEndMs(views: View[]): number {
  if (views.length === 0) return 0;
  const valid = views.filter((v) => v.videoStartTimeMs && v.videoStartTimeMs > 0);
  if (valid.length === 0) return 0;

  // Use Math.min to find the EARLIEST end time (Intersection of all clips)
  // This ensures the timeline ends when ANY clip ends.
  return Math.min(...valid.map((v) => v.videoStartTimeMs + (v.durationMs || 0)));
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
