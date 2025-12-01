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
  const [isJoined, setIsJoined] = useState(false);

  // Initialize user identity if missing
  useEffect(() => {
    if (config && (!config.user?.user_id || !config.user?.display_name)) {
      // This is a bit of a hack since updateRecordingConfig is typed for recording config
      // We might need to update useSettings to handle root config updates properly
      // For now, let's assume we can update it or we need to fix useSettings
      // Actually, looking at useSettings, updateRecordingConfig ONLY updates recording object.
      // We need a way to update the user object.
      // Let's assume for this step we just use local state if config is missing,
      // but ideally we should update the backend to support user config updates.
    }
  }, [config]);

  // Fallback to local state if config not ready, but prefer config
  // Use useMemo to ensure the random ID doesn't change on every render
  const fallbackId = React.useMemo(() => 'user-' + Math.floor(Math.random() * 10000), []);
  const userId = config?.user?.user_id || fallbackId;
  const displayName = config?.user?.display_name || 'Player';

  const { roomState, client, connectionState, error, triggerClip } = useRoom(
    isJoined ? roomId : '',
    userId,
    displayName,
    onClipStart
  );

  const handleJoin = (id: string, name: string) => {
    setRoomId(id);
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

  return <JoinRoom onJoin={handleJoin} />;
};
