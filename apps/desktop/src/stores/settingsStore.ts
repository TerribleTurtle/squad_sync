import { create } from 'zustand';
import { AppConfig } from '../types/config';
import { MonitorInfo } from '../hooks/useSettings';

interface SettingsState {
  config: AppConfig | null;
  audioDevices: string[];
  systemAudioDevices: string[];
  monitors: MonitorInfo[];
  loading: boolean;
  saving: boolean;

  setConfig: (config: AppConfig | null) => void;
  setAudioDevices: (devices: string[]) => void;
  setSystemAudioDevices: (devices: string[]) => void;
  setMonitors: (monitors: MonitorInfo[]) => void;
  setLoading: (loading: boolean) => void;
  setSaving: (saving: boolean) => void;

  // Actions to update specific parts of config
  updateUserConfig: (key: string, value: any) => void;
  updateRecordingConfig: (key: string, value: any) => void;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  config: null,
  audioDevices: [],
  systemAudioDevices: [],
  monitors: [],
  loading: true,
  saving: false,

  setConfig: (config) => set({ config }),
  setAudioDevices: (audioDevices) => set({ audioDevices }),
  setSystemAudioDevices: (systemAudioDevices) => set({ systemAudioDevices }),
  setMonitors: (monitors) => set({ monitors }),
  setLoading: (loading) => set({ loading }),
  setSaving: (saving) => set({ saving }),

  updateUserConfig: (key, value) =>
    set((state) => {
      if (!state.config) return state;
      return {
        config: {
          ...state.config,
          user: {
            ...state.config.user,
            [key]: value,
          },
        },
      };
    }),

  updateRecordingConfig: (key, value) =>
    set((state) => {
      if (!state.config) return state;
      return {
        config: {
          ...state.config,
          recording: {
            ...state.config.recording,
            [key]: value,
          },
        },
      };
    }),
}));
