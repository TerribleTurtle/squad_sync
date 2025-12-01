import PartySocket from 'partysocket';
import { WebSocket } from 'ws';

// Polyfill WebSocket for Node.js environment
global.WebSocket = WebSocket as any;

const PARTYKIT_HOST = 'localhost:1999';
const ROOM_ID = 'test-room-clip-trigger';

async function runVerification() {
  console.log('Starting Clip Trigger verification...');

  const clients: PartySocket[] = [];
  const clientCount = 3;
  const receivedMessages: Record<string, any[]> = {};

  // Create clients
  for (let i = 0; i < clientCount; i++) {
    const userId = `user-${i}`;
    receivedMessages[userId] = [];
    const client = new PartySocket({
      host: PARTYKIT_HOST,
      room: ROOM_ID,
      id: userId,
    });

    client.addEventListener('message', (event: any) => {
      try {
        const data = JSON.parse(event.data as string);
        receivedMessages[userId].push(data);
      } catch (e) {
        console.error(`Failed to parse message for ${userId}:`, event.data);
      }
    });

    clients.push(client);
  }

  // Wait for connections
  const connectedPromise = Promise.all(
    clients.map(
      (client) =>
        new Promise<void>((resolve) => {
          client.addEventListener('open', () => {
            console.log(`Client ${client.id} connected`);
            resolve();
          });
        })
    )
  );

  await connectedPromise;
  console.log('All clients connected!');

  // Join Room
  clients.forEach((client, i) => {
    client.send(
      JSON.stringify({
        type: 'JOIN_ROOM',
        roomId: ROOM_ID,
        userId: client.id,
        displayName: `Player ${i}`,
      })
    );
  });

  // Wait for joins
  await new Promise((resolve) => setTimeout(resolve, 1000));

  // Trigger Clip from Client 0
  console.log('Triggering clip from user-0...');
  clients[0].send(
    JSON.stringify({
      type: 'TRIGGER_CLIP',
      segmentCount: 60,
    })
  );

  // Wait for responses
  console.log('Waiting for START_CLIP messages...');
  await new Promise((resolve) => setTimeout(resolve, 2000));

  console.log('Closing clients...');
  clients.forEach((client) => client.close());

  // Verification Logic
  let success = true;
  for (let i = 0; i < clientCount; i++) {
    const userId = `user-${i}`;
    const messages = receivedMessages[userId];

    // Expect START_CLIP
    const startClipMsg = messages.find((m) => m.type === 'START_CLIP');
    if (!startClipMsg) {
      console.error(`❌ Client ${userId} did not receive START_CLIP`);
      success = false;
    } else {
      console.log(`✅ Client ${userId} received START_CLIP (ClipID: ${startClipMsg.clipId})`);
    }
  }

  if (success) {
    console.log('Verification complete: SUCCESS');
  } else {
    console.error('Verification complete: FAILED');
    process.exit(1);
  }
}

runVerification().catch((e) => {
  console.error(e);
  process.exit(1);
});
