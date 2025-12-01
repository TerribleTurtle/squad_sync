import PartySocket from 'partysocket';
import { ClientMessage, ServerMessage, ClientMessageSchema } from '@squadsync/shared';

export class PartyKitClient {
  private socket: PartySocket;
  private messageHandlers: Set<(msg: ServerMessage) => void> = new Set();

  constructor(host: string, roomId: string, userId: string) {
    this.socket = new PartySocket({
      host,
      room: roomId,
      id: userId,
    });

    this.socket.addEventListener('message', (event) => {
      try {
        const data = JSON.parse(event.data);
        // We trust the server, but could validate here too
        this.notifyHandlers(data as ServerMessage);
      } catch (e) {
        console.error('Failed to parse server message', e);
      }
    });
  }

  public send(message: ClientMessage) {
    // Validate outgoing messages
    const result = ClientMessageSchema.safeParse(message);
    if (!result.success) {
      console.error('Invalid client message', result.error);
      return;
    }
    this.socket.send(JSON.stringify(message));
  }

  public onMessage(handler: (msg: ServerMessage) => void) {
    this.messageHandlers.add(handler);
    return () => {
      this.messageHandlers.delete(handler);
    };
  }

  private notifyHandlers(msg: ServerMessage) {
    this.messageHandlers.forEach((handler) => handler(msg));
  }

  public get id(): string {
    return this.socket.id;
  }

  public close() {
    this.socket.close();
  }
}
