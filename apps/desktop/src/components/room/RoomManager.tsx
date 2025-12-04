import React, { useState, useEffect } from 'react';
import { useRoom } from '../../hooks/useRoom';
import { useSettings } from '../../hooks/useSettings';
import { logger } from '../../lib/logger';
import { JoinRoom } from './JoinRoom';
import { SquadList } from './SquadList';
import { Loader2, AlertCircle } from 'lucide-react';

interface RoomManagerProps {
  onClipStart?: (timestamp: number, uploadUrl?: string, clipId?: string) => Promise<void>;
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
    async (timestamp, uploadUrl, clipId) => {
      if (onClipStart) {
        await onClipStart(timestamp, uploadUrl, clipId);

        // If we have a clipId and uploadUrl, it means we attempted an upload.
        // Notify server to verify.
        if (clipId && uploadUrl && client) {
          logger.info('📤 Sending UPLOAD_COMPLETE for', clipId);
          client.send({
            type: 'UPLOAD_COMPLETE',
            clipId,
          });
        }
      }
    }
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

  if (isJoined) {
    if (error) {
      return (
        <div className="w-full max-w-sm mx-auto p-6 bg-slate-900/50 backdrop-blur-xl rounded-2xl border border-white/10 shadow-xl flex flex-col items-center justify-center min-h-[200px]">
          <AlertCircle className="text-red-400 mb-4" size={32} />
          <h3 className="text-lg font-bold text-white">Connection Failed</h3>
          <p className="text-red-400 text-sm mt-2 text-center">{error}</p>
          <button
            onClick={handleLeave}
            className="mt-6 px-4 py-2 bg-slate-800 hover:bg-slate-700 text-white rounded-lg transition-colors"
          >
            Back to Menu
          </button>
        </div>
      );
    }

    if (!roomState) {
      return (
        <div className="w-full max-w-sm mx-auto p-6 bg-slate-900/50 backdrop-blur-xl rounded-2xl border border-white/10 shadow-xl flex flex-col items-center justify-center min-h-[200px]">
          <Loader2 className="animate-spin text-indigo-500 mb-4" size={32} />
          <h3 className="text-lg font-bold text-white">Connecting to Squad...</h3>
          <p className="text-slate-400 text-sm mt-2">Establishing secure connection</p>
          <button
            onClick={handleLeave}
            className="mt-6 text-sm text-slate-500 hover:text-slate-300 transition-colors"
          >
            Cancel
          </button>
        </div>
      );
    }

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
