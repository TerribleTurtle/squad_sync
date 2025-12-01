import { useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useRecordingStore } from '../stores/recordingStore';
import { useToastStore } from '../stores/toastStore';
import { REPLAY_BUFFER_DELAY, CLIP_SAVE_DELAY } from '@squadsync/shared';

export function useRecorder() {
  const { status, isReplayActive, isBuffering, setStatus, setReplayActive, setBuffering } =
    useRecordingStore();

  const { showToast } = useToastStore();

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Alt+F10 to save clip
      if (e.altKey && e.key === 'F10') {
        if (isReplayActive) {
          saveReplay();
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isReplayActive]);

  const enableReplay = useCallback(async () => {
    try {
      setBuffering(true);
      setStatus('Initializing Buffer...');
      await invoke('enable_replay');

      // Simulate buffering delay for UX
      setTimeout(() => {
        setBuffering(false);
        setReplayActive(true);
        setStatus('Replay Buffer Active');
        showToast('Replay Buffer Enabled', 'success');
      }, REPLAY_BUFFER_DELAY);
    } catch (e) {
      setBuffering(false);
      setStatus(`Error: ${e}`);
      showToast(`Error: ${e}`, 'error');
    }
  }, [setBuffering, setStatus, setReplayActive, showToast]);

  const disableReplay = useCallback(async () => {
    try {
      await invoke('disable_replay');
      setReplayActive(false);
      setStatus('Replay Buffer Disabled');
      showToast('Replay Buffer Disabled', 'success');
    } catch (e) {
      setStatus(`Error: ${e}`);
      showToast(`Error: ${e}`, 'error');
    }
  }, [setReplayActive, setStatus, showToast]);

  const saveReplay = useCallback(
    async (timestamp?: number) => {
      try {
        setStatus('Saving Clip...');
        // Pass timestamp to backend (maps to trigger_timestamp in Rust)
        await invoke('save_replay', { trigger_timestamp: timestamp });
        setStatus(`Clip Saved!`);
        showToast('Clip Saved Successfully!', 'success');
        setTimeout(() => setStatus('Replay Buffer Active'), CLIP_SAVE_DELAY);
      } catch (e) {
        setStatus(`Error saving: ${e}`);
        showToast(`Error saving: ${e}`, 'error');
      }
    },
    [setStatus, showToast]
  );

  return {
    status,
    isReplayActive,
    isBuffering,
    enableReplay,
    disableReplay,
    saveReplay,
  };
}
