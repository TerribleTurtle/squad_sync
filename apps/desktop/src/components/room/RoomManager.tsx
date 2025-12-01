import React, { useState } from 'react';
import { useRoom } from '../../hooks/useRoom';
import { JoinRoom } from './JoinRoom';
import { SquadList } from './SquadList';

export const RoomManager: React.FC = () => {
  const [roomId, setRoomId] = useState<string>('');
  const [displayName, setDisplayName] = useState<string>('');
  const [isJoined, setIsJoined] = useState(false);

  // Generate a stable User ID on mount
  const [userId] = useState(() => 'user-' + Math.floor(Math.random() * 10000));

  // Only connect when we have both ID and Name
  const { roomState, client } = useRoom(isJoined ? roomId : '', userId, displayName);

  const handleJoin = (id: string, name: string) => {
    setRoomId(id);
    setDisplayName(name);
    setIsJoined(true);
  };

  const handleLeave = () => {
    setIsJoined(false);
    setRoomId('');
    // client.close() is handled by useRoom when roomId becomes empty
  };

  if (isJoined && roomState) {
    return (
      <SquadList roomState={roomState} currentUserId={client?.id || ''} onLeave={handleLeave} />
    );
  }

  return <JoinRoom onJoin={handleJoin} />;
};
