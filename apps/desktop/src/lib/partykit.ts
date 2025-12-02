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

    this.socket.addEventListener('open', () => {
      this.notifyConnectionHandlers('connect');
    });

    this.socket.addEventListener('close', () => {
      this.notifyConnectionHandlers('disconnect');
    });

    this.socket.addEventListener('error', (err) => {
      console.error('PartySocket error:', err);
      this.notifyConnectionHandlers('error', err);
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

  public onConnect(handler: () => void) {
    const handlerObj = { type: 'connect' as const, handler };
    this.connectionHandlers.add(handlerObj);
    // If already connected, fire immediately
    if (this.socket.readyState === 1) {
      // WebSocket.OPEN
      handler();
    }
    return () => {
      this.connectionHandlers.delete(handlerObj);
    };
  }

  public onDisconnect(handler: () => void) {
    const handlerObj = { type: 'disconnect' as const, handler };
    this.connectionHandlers.add(handlerObj);
    return () => {
      this.connectionHandlers.delete(handlerObj);
    };
  }

  public onError(handler: (err: Event) => void) {
    const handlerObj = { type: 'error' as const, handler };
    this.connectionHandlers.add(handlerObj);
    return () => {
      this.connectionHandlers.delete(handlerObj);
    };
  }

  private notifyHandlers(msg: ServerMessage) {
    this.messageHandlers.forEach((handler) => handler(msg));
  }

  private connectionHandlers: Set<{
    type: 'connect' | 'disconnect' | 'error';
    handler: Function;
  }> = new Set();

  private notifyConnectionHandlers(type: 'connect' | 'disconnect' | 'error', data?: any) {
    this.connectionHandlers.forEach((h) => {
      if (h.type === type) {
        h.handler(data);
      }
    });
  }

  public get id(): string {
    return this.socket.id;
  }

  public close() {
    this.socket.close();
  }
}
