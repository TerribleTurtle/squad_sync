import { z } from 'zod';

export const JoinRoomSchema = z.object({
  type: z.literal('JOIN_ROOM'),
  roomId: z.string(),
  userId: z.string(),
  displayName: z.string(),
});

export const LeaveRoomSchema = z.object({
  type: z.literal('LEAVE_ROOM'),
});

export const TimeSyncRequestSchema = z.object({
  type: z.literal('TIME_SYNC_REQUEST'),
  clientTime: z.number(),
});

export const TriggerClipSchema = z.object({
  type: z.literal('TRIGGER_CLIP'),
  segmentCount: z.number().default(60),
});

export const UploadCompleteSchema = z.object({
  type: z.literal('UPLOAD_COMPLETE'),
  clipId: z.string(),
  key: z.string().optional(),
  videoStartTimeMs: z.number(),
  durationMs: z.number(),
});

export const RequestUploadUrlSchema = z.object({
  type: z.literal('REQUEST_UPLOAD_URL'),
  clipId: z.string(),
});

export const ClientMessageSchema = z.discriminatedUnion('type', [
  JoinRoomSchema,
  LeaveRoomSchema,
  TimeSyncRequestSchema,
  TriggerClipSchema,
  UploadCompleteSchema,
  RequestUploadUrlSchema,
]);
