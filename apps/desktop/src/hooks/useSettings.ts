import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { AppConfig } from '../types/config';
import { useToastStore } from '../stores/toastStore';
import { useSettingsStore } from '../stores/settingsStore';

export interface MonitorInfo {
  id: number;
  name: string;
  width: number;
  height: number;
  is_primary: boolean;
}

export function useSettings() {
  const store = useSettingsStore();
  const { showToast } = useToastStore();

  useEffect(() => {
    // Only load if we haven't loaded config yet
    if (!store.config && store.loading) {
      loadData();
    }
  }, []);

  async function loadData() {
    try {
      store.setLoading(true);
      const [loadedConfig, devices, systemDevices, monitorList] = await Promise.all([
        invoke<AppConfig>('get_config'),
        invoke<string[]>('get_audio_devices'),
        invoke<string[]>('get_system_audio_devices'),
        invoke<MonitorInfo[]>('get_monitors'),
      ]);

      store.setConfig(loadedConfig);
      store.setAudioDevices(devices);
      store.setSystemAudioDevices(systemDevices);
      store.setMonitors(monitorList);
    } catch (e) {
      console.error('Failed to load settings:', e);
    } finally {
      store.setLoading(false);
    }
  }

  async function saveSettings() {
    if (!store.config) return;

    store.setSaving(true);
    try {
      await invoke('update_config', { newConfig: store.config });
      showToast('Settings saved successfully!', 'success');
    } catch (e) {
      console.error('Failed to save settings:', e);
      showToast(`Error saving settings: ${e}`, 'error');
    } finally {
      store.setSaving(false);
    }
  }

  return {
    config: store.config,
    audioDevices: store.audioDevices,
    systemAudioDevices: store.systemAudioDevices,
    monitors: store.monitors,
    loading: store.loading,
    saving: store.saving,
    saveSettings,
    updateRecordingConfig: store.updateRecordingConfig,
    updateUserConfig: store.updateUserConfig,
  };
}
