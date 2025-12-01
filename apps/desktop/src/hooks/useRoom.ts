import { useEffect, useState, useRef } from 'react';
import { PartyKitClient } from '../lib/partykit';
import { RoomState, RoomMember } from '@squadsync/shared';

// TODO: Move to config
const PARTYKIT_HOST = import.meta.env.VITE_PARTYKIT_HOST || 'localhost:1999';

export function useRoom(roomId: string, userId: string, displayName: string) {
  const [roomState, setRoomState] = useState<RoomState | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const clientRef = useRef<PartyKitClient | null>(null);

  useEffect(() => {
    if (!roomId || !userId) return;

    const client = new PartyKitClient(PARTYKIT_HOST, roomId, userId);
    clientRef.current = client;
    setIsConnected(true);

    // Join the room
    client.send({
      type: 'JOIN_ROOM',
      roomId,
      userId,
      displayName,
    });

    // Listen for messages
    const unsubscribe = client.onMessage((msg) => {
      switch (msg.type) {
        case 'ROOM_STATE':
          setRoomState(msg.state);
          break;
        case 'MEMBER_JOINED':
          setRoomState((prev: RoomState | null) => {
            if (!prev) return null;
            return {
              ...prev,
              members: [...prev.members, msg.member],
            };
          });
          break;
        case 'MEMBER_LEFT':
          setRoomState((prev: RoomState | null) => {
            if (!prev) return null;
            return {
              ...prev,
              members: prev.members.filter((m: RoomMember) => m.userId !== msg.userId),
            };
          });
          break;
      }
    });

    return () => {
      unsubscribe();
      client.close();
      setIsConnected(false);
      clientRef.current = null;
    };
  }, [roomId, userId, displayName]);

  return {
    roomState,
    isConnected,
    client: clientRef.current,
  };
}
