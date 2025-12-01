import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { AppConfig } from '../types/config';
import { useToastStore } from '../stores/toastStore';

export interface MonitorInfo {
  id: number;
  name: string;
  width: number;
  height: number;
  is_primary: boolean;
}

export function useSettings() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [audioDevices, setAudioDevices] = useState<string[]>([]);
  const [systemAudioDevices, setSystemAudioDevices] = useState<string[]>([]);
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  const { showToast } = useToastStore();

  useEffect(() => {
    loadData();
  }, []);

  async function loadData() {
    try {
      const [loadedConfig, devices, systemDevices, monitorList] = await Promise.all([
        invoke<AppConfig>('get_config'),
        invoke<string[]>('get_audio_devices'),
        invoke<string[]>('get_system_audio_devices'),
        invoke<MonitorInfo[]>('get_monitors'),
      ]);

      setConfig(loadedConfig);
      setAudioDevices(devices);
      setSystemAudioDevices(systemDevices);
      setMonitors(monitorList);
    } catch (e) {
      console.error('Failed to load settings:', e);
    } finally {
      setLoading(false);
    }
  }

  async function saveSettings() {
    if (!config) return;

    setSaving(true);
    try {
      await invoke('update_config', { newConfig: config });
      showToast('Settings saved successfully!', 'success');
    } catch (e) {
      console.error('Failed to save settings:', e);
      showToast(`Error saving settings: ${e}`, 'error');
    } finally {
      setSaving(false);
    }
  }

  function updateRecordingConfig(key: string, value: any) {
    if (!config) return;

    const newConfig = {
      ...config,
      recording: {
        ...config.recording,
        [key]: value,
      },
    };

    setConfig(newConfig);
  }

  function updateUserConfig(key: string, value: any) {
    if (!config) return;

    const newConfig = {
      ...config,
      user: {
        ...config.user,
        [key]: value,
      },
    };

    setConfig(newConfig);
  }

  return {
    config,
    audioDevices,
    systemAudioDevices,
    monitors,
    loading,
    saving,
    saveSettings,
    updateRecordingConfig,
    updateUserConfig,
  };
}
