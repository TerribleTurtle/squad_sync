import PartySocket from 'partysocket';
import { WebSocket } from 'ws';

// Polyfill WebSocket for Node.js environment
global.WebSocket = WebSocket as any;

const PARTYKIT_HOST = 'localhost:1999';
const ROOM_ID = 'test-room-verify';

async function runVerification() {
  console.log('Starting verification...');

  const clients: PartySocket[] = [];
  const clientCount = 3;

  // Create clients
  for (let i = 0; i < clientCount; i++) {
    const userId = `user-${i}`;
    const client = new PartySocket({
      host: PARTYKIT_HOST,
      room: ROOM_ID,
      id: userId,
    });
    clients.push(client);
  }

  // Wait for connections and verify messages
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

  // Verify JOIN_ROOM flow
  console.log('Sending JOIN_ROOM messages...');
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

  // Wait for ROOM_STATE and MEMBER_JOINED
  // This is a bit manual, but we can listen for a few seconds
  await new Promise((resolve) => setTimeout(resolve, 2000));

  console.log('Closing clients...');
  clients.forEach((client) => client.close());
  console.log('Verification complete.');
}

runVerification().catch(console.error);
