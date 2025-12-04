import { useEffect, useState, useRef } from 'react';
import { PartyKitClient } from '../lib/partykit';
import { RoomState, RoomMember } from '@squadsync/shared';
import { PARTYKIT_HOST } from '../lib/constants';
import { useToastStore } from '../stores/toastStore';

export type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error';

export function useRoom(
  roomId: string,
  userId: string,
  displayName: string,
  onClipStart?: (timestamp: number, uploadUrl?: string, clipId?: string) => void
) {
  const [roomState, setRoomState] = useState<RoomState | null>(null);
  const [connectionState, setConnectionState] = useState<ConnectionState>('disconnected');
  const [error, setError] = useState<string | null>(null);
  const clientRef = useRef<PartyKitClient | null>(null);
  const onClipStartRef = useRef(onClipStart);
  const pendingClipRef = useRef<{ clipId: string; referenceTime: number } | null>(null);

  useEffect(() => {
    onClipStartRef.current = onClipStart;
  }, [onClipStart]);

  useEffect(() => {
    if (!roomId || !userId) {
      console.info('⚠️ useRoom: Missing roomId or userId', { roomId, userId });
      setConnectionState('disconnected');
      return;
    }

    console.info('🔄 useRoom: Initializing connection', { roomId, userId });
    setConnectionState('connecting');
    setError(null);

    const client = new PartyKitClient(PARTYKIT_HOST, roomId, userId);
    clientRef.current = client;

    // Connection handlers
    const unsubConnect = client.onConnect(() => {
      console.info('✅ useRoom: Connected');
      setConnectionState('connected');
      setError(null);
    });

    const unsubDisconnect = client.onDisconnect(() => {
      console.info('❌ useRoom: Disconnected');
      setConnectionState('disconnected');
    });

    const unsubError = client.onError((err) => {
      console.error('🔥 useRoom: Connection error', err);
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
          console.info('🎥 START_CLIP received:', msg);
          // Store reference time and request upload URL
          pendingClipRef.current = { clipId: msg.clipId, referenceTime: msg.referenceTime };

          if (clientRef.current) {
            console.info('📤 Requesting upload URL for clip:', msg.clipId);
            clientRef.current.send({
              type: 'REQUEST_UPLOAD_URL',
              clipId: msg.clipId,
            });
          }
          break;
        case 'UPLOAD_URL_GRANTED':
          console.info('✅ UPLOAD_URL_GRANTED received:', msg);
          if (pendingClipRef.current && pendingClipRef.current.clipId === msg.clipId) {
            // Now trigger the actual recording/upload
            onClipStartRef.current?.(
              pendingClipRef.current.referenceTime,
              msg.uploadUrl,
              msg.clipId
            );
            pendingClipRef.current = null;
          } else {
            console.warn('⚠️ Received UPLOAD_URL_GRANTED for unknown or stale clip:', msg.clipId);
          }
          break;
        case 'ERROR':
          console.error('❌ Signaling Error:', msg);
          setError(`[${msg.code}] ${msg.message}`);
          setConnectionState('error');
          useToastStore.getState().showToast(`Signaling Error: ${msg.message}`, 'error');
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
  }, [roomId, userId]); // Only reconnect if room or user ID changes

  // Separate effect for joining/updating user info
  useEffect(() => {
    if (connectionState === 'connected' && clientRef.current) {
      console.info('Sending JOIN_ROOM with name:', displayName);
      clientRef.current.send({
        type: 'JOIN_ROOM',
        roomId,
        userId,
        displayName,
      });
    }
  }, [connectionState, roomId, userId, displayName]);

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
