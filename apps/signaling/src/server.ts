import type * as Party from 'partykit/server';
import { RoomHandler } from './handlers/room';
import { SyncHandler } from './handlers/sync';
import { ClientMessageSchema, ClientMessage } from '@squadsync/shared';

export default class Server implements Party.Server {
  private roomHandler: RoomHandler;
  private syncHandler: SyncHandler;

  // Map connection ID to User ID for disconnect handling
  private connToUser: Map<string, string> = new Map();

  constructor(readonly room: Party.Room) {
    this.roomHandler = new RoomHandler(room);
    this.syncHandler = new SyncHandler();
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
          // TODO: Implement clip triggering
          console.log('Clip triggered by', sender.id);
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
