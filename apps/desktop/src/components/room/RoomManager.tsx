import React, { useState, useEffect } from 'react';
import { useRoom } from '../../hooks/useRoom';
import { useSettings } from '../../hooks/useSettings';
import { JoinRoom } from './JoinRoom';
import { SquadList } from './SquadList';

interface RoomManagerProps {
  onClipStart?: (timestamp: number) => void;
}

export const RoomManager: React.FC<RoomManagerProps> = ({ onClipStart }) => {
  const { config, updateUserConfig, saveSettings } = useSettings();
  const [roomId, setRoomId] = useState<string>('');
  const [displayName, setDisplayName] = useState('');
  const [isJoined, setIsJoined] = useState(false);

  // Initialize user identity from config
  useEffect(() => {
    if (config?.user?.display_name) {
      setDisplayName(config.user.display_name);
    }
  }, [config]);

  // Fallback to local state if config not ready, but prefer config
  // Use useMemo to ensure the random ID doesn't change on every render
  const fallbackId = React.useMemo(() => 'user-' + Math.floor(Math.random() * 10000), []);
  const userId = config?.user?.user_id || fallbackId;
  const effectiveDisplayName = displayName || 'Player';

  const { roomState, client, connectionState, error, triggerClip } = useRoom(
    isJoined ? roomId : '',
    userId,
    effectiveDisplayName,
    onClipStart
  );

  const handleJoin = (id: string, name: string) => {
    setRoomId(id);
    setDisplayName(name);
    if (config) {
      updateUserConfig('display_name', name);
      // If we don't have a user_id, generate one and save it too
      if (!config.user?.user_id) {
        updateUserConfig('user_id', userId);
      }
      // Save immediately
      setTimeout(() => saveSettings(), 0);
    }
    setIsJoined(true);
  };

  const handleLeave = () => {
    setIsJoined(false);
    setRoomId('');
  };

  if (isJoined && roomState) {
    return (
      <SquadList
        roomState={roomState}
        currentUserId={client?.id || ''}
        connectionState={connectionState}
        error={error}
        onLeave={handleLeave}
        onTriggerClip={triggerClip}
      />
    );
  }

  return (
    <JoinRoom
      onJoin={handleJoin}
      initialDisplayName={displayName}
      onDisplayNameChange={(name) => updateUserConfig('display_name', name)}
    />
  );
};
