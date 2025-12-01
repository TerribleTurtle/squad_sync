import { create } from 'zustand';

interface RecordingState {
  status: string;
  isReplayActive: boolean;
  isBuffering: boolean;
  setStatus: (status: string) => void;
  setReplayActive: (isActive: boolean) => void;
  setBuffering: (isBuffering: boolean) => void;
}

export const useRecordingStore = create<RecordingState>((set) => ({
  status: 'Ready',
  isReplayActive: false,
  isBuffering: false,
  setStatus: (status) => set({ status }),
  setReplayActive: (isReplayActive) => set({ isReplayActive }),
  setBuffering: (isBuffering) => set({ isBuffering }),
}));
