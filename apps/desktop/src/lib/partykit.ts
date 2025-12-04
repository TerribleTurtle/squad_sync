import PartySocket from 'partysocket';
import { ClientMessage, ServerMessage, ClientMessageSchema } from '@squadsync/shared';

export class PartyKitClient {
  private socket: PartySocket;
  private messageHandlers: Set<(msg: ServerMessage) => void> = new Set();

  constructor(host: string, roomId: string, userId: string) {
    // Force WS (insecure) for localhost and IP addresses to avoid SSL issues
    const isIP = /^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}/.test(host);
    const protocol = host.includes('localhost') || isIP ? 'ws' : undefined;

    const debugInfo = {
      host,
      roomId,
      userId,
      isIP,
      protocol,
      fullUrl: `${protocol || 'wss'}://${host}/party/${roomId}`,
    };

    console.info('🔌 PartyKitClient initializing:', debugInfo);

    // Show debug alert (remove after debugging)
    // alert(
    //   `DEBUG: PartyKit Connection\nHost: ${host}\nProtocol: ${protocol || 'wss'}\nFull URL: ${debugInfo.fullUrl}\nIs IP: ${isIP}`
    // );

    this.socket = new PartySocket({
      host,
      room: roomId,
      id: userId,
      protocol: protocol as 'ws' | 'wss' | undefined,
    });

    this.socket.addEventListener('open', () => {
      console.info('✅ PartySocket connected');
      this.notifyConnectionHandlers('connect');
    });

    this.socket.addEventListener('close', (event) => {
      console.info('❌ PartySocket disconnected:', event);
      this.notifyConnectionHandlers('disconnect');
    });

    this.socket.addEventListener('error', (err) => {
      console.error('🔥 PartySocket error:', err);
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

  private connectionHandlers: Set<
    | { type: 'connect'; handler: () => void }
    | { type: 'disconnect'; handler: () => void }
    | { type: 'error'; handler: (err: Event) => void }
  > = new Set();

  private notifyConnectionHandlers(type: 'connect' | 'disconnect' | 'error', data?: unknown) {
    this.connectionHandlers.forEach((h) => {
      if (h.type === type) {
        if (h.type === 'error') {
          h.handler(data as Event);
        } else {
          h.handler();
        }
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
