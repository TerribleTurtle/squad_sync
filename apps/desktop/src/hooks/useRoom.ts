import { useEffect, useState, useRef } from 'react';
import { PartyKitClient } from '../lib/partykit';
import { RoomState, RoomMember } from '@squadsync/shared';
import { PARTYKIT_HOST } from '../lib/constants';

export type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error';

export function useRoom(
  roomId: string,
  userId: string,
  displayName: string,
  onClipStart?: (timestamp: number) => void
) {
  const [roomState, setRoomState] = useState<RoomState | null>(null);
  const [connectionState, setConnectionState] = useState<ConnectionState>('disconnected');
  const [error, setError] = useState<string | null>(null);
  const clientRef = useRef<PartyKitClient | null>(null);
  const onClipStartRef = useRef(onClipStart);

  useEffect(() => {
    onClipStartRef.current = onClipStart;
  }, [onClipStart]);

  useEffect(() => {
    if (!roomId || !userId) {
      setConnectionState('disconnected');
      return;
    }

    setConnectionState('connecting');
    setError(null);

    const client = new PartyKitClient(PARTYKIT_HOST, roomId, userId);
    clientRef.current = client;

    // Connection handlers
    const unsubConnect = client.onConnect(() => {
      setConnectionState('connected');
      setError(null);

      // Join the room on connect
      client.send({
        type: 'JOIN_ROOM',
        roomId,
        userId,
        displayName,
      });
    });

    const unsubDisconnect = client.onDisconnect(() => {
      setConnectionState('disconnected');
    });

    const unsubError = client.onError((err) => {
      setConnectionState('error');
      setError('Connection error occurred');
      console.error('PartyKit error:', err);
    });

    // Listen for messages
    const unsubMessage = client.onMessage((msg) => {
      switch (msg.type) {
        case 'ROOM_STATE':
          setRoomState(msg.state);
          break;
        case 'MEMBER_JOINED':
          setRoomState((prev: RoomState | null) => {
            if (!prev) return null;
            // Check if member exists
            const existingIndex = prev.members.findIndex((m) => m.userId === msg.member.userId);

            if (existingIndex !== -1) {
              // Update existing member
              const newMembers = [...prev.members];
              newMembers[existingIndex] = msg.member;
              return {
                ...prev,
                members: newMembers,
              };
            }

            // Add new member
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
        case 'START_CLIP':
          console.log('🎥 START_CLIP received:', msg);
          // Use the ref here
          onClipStartRef.current?.(msg.referenceTime);
          break;
      }
    });

    return () => {
      unsubConnect();
      unsubDisconnect();
      unsubError();
      unsubMessage();
      client.close();
      setConnectionState('disconnected');
      clientRef.current = null;
    };
  }, [roomId, userId, displayName]); // Removed onClipStart from dependencies

  const triggerClip = () => {
    if (clientRef.current) {
      clientRef.current.send({
        type: 'TRIGGER_CLIP',
        segmentCount: 60, // Default to 60s
      });
    }
  };

  return {
    roomState,
    connectionState,
    error,
    client: clientRef.current,
    triggerClip,
  };
}
