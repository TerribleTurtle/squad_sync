import type * as Party from 'partykit/server';
import { RoomHandler } from './handlers/room';
import { SyncHandler } from './handlers/sync';
import { ClientMessageSchema, ClientMessage } from '@squadsync/shared';

import { RateLimiter } from './lib/ratelimit';

export default class Server implements Party.Server {
  private roomHandler: RoomHandler;
  private syncHandler: SyncHandler;
  private rateLimiter: RateLimiter;

  // Map connection ID to User ID for disconnect handling
  private connToUser: Map<string, string> = new Map();

  constructor(readonly room: Party.Room) {
    this.roomHandler = new RoomHandler(room);
    this.syncHandler = new SyncHandler();
    this.rateLimiter = new RateLimiter();
  }

  async onConnect(conn: Party.Connection, ctx: Party.ConnectionContext) {
    console.log(`Connected: ${conn.id} to room ${this.room.id}`);
  }

  async onMessage(message: string, sender: Party.Connection) {
    try {
      const parsed = JSON.parse(message);
      const result = ClientMessageSchema.safeParse(parsed);

      if (!result.success) {
        console.error('Invalid message:', result.error);
        return;
      }

      const msg = result.data as ClientMessage;

      // Check Rate Limit
      if (!this.rateLimiter.checkLimit(sender.id, msg.type as any)) {
        console.warn(`Rate limit exceeded for ${sender.id} on ${msg.type}`);
        sender.send(
          JSON.stringify({
            type: 'ERROR',
            code: 'RATE_LIMITED',
            message: 'You are sending too many requests. Please slow down.',
          })
        );
        return;
      }

      switch (msg.type) {
        case 'JOIN_ROOM':
          this.connToUser.set(sender.id, msg.userId);
          await this.roomHandler.handleJoin(sender, msg);
          break;
        case 'LEAVE_ROOM':
          const userId = this.connToUser.get(sender.id);
          if (userId) {
            await this.roomHandler.handleLeave(sender, userId);
            this.connToUser.delete(sender.id);
          }
          break;
        case 'TIME_SYNC_REQUEST':
          this.syncHandler.handleTimeSync(sender, msg);
          break;
        case 'TRIGGER_CLIP':
          const clipId = crypto.randomUUID();
          const startClipMsg = {
            type: 'START_CLIP',
            clipId,
            segmentCount: msg.segmentCount || 60, // Default to 60s if not specified
            referenceTime: Date.now(),
            uploadUrl: '', // TODO: Generate presigned URL here later
          };
          this.room.broadcast(JSON.stringify(startClipMsg));
          console.log(`Clip ${clipId} triggered by ${sender.id}`);
          break;
        case 'UPLOAD_COMPLETE':
          // TODO: Implement upload completion
          console.log('Upload complete from', sender.id);
          break;
      }
    } catch (e) {
      console.error('Error processing message:', e);
    }
  }

  async onClose(conn: Party.Connection) {
    console.log(`Disconnected: ${conn.id}`);
    const userId = this.connToUser.get(conn.id);
    if (userId) {
      await this.roomHandler.handleLeave(conn, userId);
      this.connToUser.delete(conn.id);
    }
  }
}

Server satisfies Party.Worker;
