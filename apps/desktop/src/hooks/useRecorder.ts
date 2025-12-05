import { useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
// import { readFile } from '@tauri-apps/plugin-fs'; // Removed
// import { fetch } from '@tauri-apps/plugin-http'; // Removed
import { useRecordingStore } from '../stores/recordingStore';
import { useToastStore } from '../stores/toastStore';
import { REPLAY_BUFFER_DELAY, CLIP_SAVE_DELAY } from '@squadsync/shared';
import { logger } from '../lib/logger';

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
    async (timestamp?: number, uploadUrl?: string) => {
      try {
        setStatus('Saving Clip...');
        // Pass timestamp to backend (maps to trigger_timestamp in Rust)
        // Response is now a SavedReplay object
        interface SavedReplay {
          file_path: string;
          duration_ms: number;
          start_time_utc_ms: number | null;
          version: number;
        }

        const savedReplay = await invoke<SavedReplay>('save_replay', {
          trigger_timestamp: timestamp,
        });
        const filePath = savedReplay.file_path;

        setStatus(`Clip Saved!`);
        showToast('Clip Saved Successfully!', 'success');

        if (uploadUrl && filePath) {
          // FAIL HARD POLICY: Check for valid timestamp
          if (savedReplay.start_time_utc_ms === null) {
            logger.error('❌ Upload Aborted: Missing UTC Start Time');
            showToast('Upload Skipped: Unable to sync clip (Missing Timestamp)', 'error');
            // We do NOT throw here, we just skip upload. Local file is safe.
          } else {
            try {
              setStatus('Uploading Clip...');
              logger.info(`📤 Uploading ${filePath} to ${uploadUrl}`);

              // Use Rust backend for upload (Streaming)
              await invoke('upload_clip_to_url', {
                filePath,
                uploadUrl,
              });

              logger.info('✅ Upload Successful');

              setStatus('Upload Complete!');
              showToast('Clip Uploaded Successfully!', 'success');

              // TODO: In Phase 3, we will send start_time_utc_ms to signaling here
            } catch (uploadErr) {
              const errorMessage =
                uploadErr instanceof Error ? uploadErr.message : String(uploadErr);
              logger.error('Upload Error:', errorMessage);
              showToast(`Upload Failed: ${errorMessage}`, 'error');
              // Don't fail the whole operation, just the upload.
              // Return null so we don't send UPLOAD_COMPLETE.
              return null;
            }
          }
        }

        setTimeout(() => setStatus('Replay Buffer Active'), CLIP_SAVE_DELAY);
        return savedReplay.start_time_utc_ms;
      } catch (e) {
        setStatus(`Error saving: ${e}`);
        showToast(`Error saving: ${e}`, 'error');
        return null;
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
