import type * as Party from 'partykit/server';
import { TimeSyncRequestMessage, TimeSyncResponseMessage } from '@squadsync/shared';

export class SyncHandler {
  handleTimeSync(conn: Party.Connection, message: TimeSyncRequestMessage): void {
    const serverReceive = Date.now();
    const response: TimeSyncResponseMessage = {
      type: 'TIME_SYNC_RESPONSE',
      clientTime: message.clientTime,
      serverReceive,
      serverSend: Date.now(), // Will be slightly after receive
    };
    conn.send(JSON.stringify(response));
  }
}
