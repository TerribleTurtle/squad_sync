import { describe, it, expect } from 'vitest';
import {
  JoinRoomSchema,
  LeaveRoomSchema,
  TimeSyncRequestSchema,
  TriggerClipSchema,
  UploadCompleteSchema,
  RequestUploadUrlSchema,
  ClientMessageSchema,
} from './validation';

describe('Validation Schemas', () => {
  describe('JoinRoomSchema', () => {
    it('validates a correct join room message', () => {
      const valid = {
        type: 'JOIN_ROOM',
        roomId: 'room-123',
        userId: 'user-456',
        displayName: 'Player One',
      };
      expect(JoinRoomSchema.safeParse(valid).success).toBe(true);
    });

    it('fails on missing fields', () => {
      const invalid = {
        type: 'JOIN_ROOM',
        roomId: 'room-123',
      };
      expect(JoinRoomSchema.safeParse(invalid).success).toBe(false);
    });
  });

  describe('ClientMessageSchema', () => {
    it('validates a discriminated union', () => {
      const msg = {
        type: 'TRIGGER_CLIP',
        segmentCount: 30,
      };
      const result = ClientMessageSchema.safeParse(msg);
      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data.type).toBe('TRIGGER_CLIP');
      }
    });

    it('fails on unknown type', () => {
      const msg = {
        type: 'UNKNOWN_TYPE',
      };
      expect(ClientMessageSchema.safeParse(msg).success).toBe(false);
    });
  });
});
