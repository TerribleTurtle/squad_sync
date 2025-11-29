import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppConfig } from "../types/config";

export function Settings() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [audioDevices, setAudioDevices] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  async function loadData() {
    try {
      const [loadedConfig, devices] = await Promise.all([
        invoke<AppConfig>("get_config"),
        invoke<string[]>("get_audio_devices"),
      ]);
      
      // Force update bitrate to new standards if it looks like an old config
      // This ensures the user gets the new 30M/15M defaults without toggling
      if (loadedConfig && loadedConfig.recording) {
        const res = loadedConfig.recording.resolution || "native";
        const fps = loadedConfig.recording.framerate || 60;
        let targetBitrate = "15M";
        
        if (res === "native") {
            targetBitrate = fps === 60 ? "30M" : "15M";
        } else if (res === "1920x1080") {
            targetBitrate = fps === 60 ? "20M" : "10M"; // Bumped to 20M
        } else if (res === "1280x720") {
            targetBitrate = fps === 60 ? "12M" : "6M";  // Bumped to 12M
        }

        // Apply update if current bitrate is lower than target (or if we just want to enforce it)
        // We'll just enforce it to be safe for this "Master Quality" push
        loadedConfig.recording.bitrate = targetBitrate;
        console.log(`Auto-upgraded bitrate to ${targetBitrate}`);
      }

      setConfig(loadedConfig);
      setAudioDevices(devices);
    } catch (e) {
      console.error("Failed to load settings:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleSave(e: React.FormEvent) {
    e.preventDefault();
    if (!config) return;

    setSaving(true);
    try {
      await invoke("update_config", { newConfig: config });
      alert("Settings saved!");
    } catch (e) {
      console.error("Failed to save settings:", e);
      alert(`Error saving settings: ${e}`);
    } finally {
      setSaving(false);
    }
  }

  function updateRecordingConfig(key: string, value: any) {
    if (!config) return;
    
    let newConfig = {
      ...config,
      recording: {
        ...config.recording,
        [key]: value,
      },
    };

    // Auto-Calculate Bitrate based on Resolution & FPS
    // Heuristic (Master Quality):
    // Native (1440p) @ 60 -> 30 Mbps (Crystal Clear)
    // 1080p @ 60 -> 15 Mbps (High Quality)
    // 720p @ 60 -> 10 Mbps (Good Quality)
    // (Halve for 30fps)
    if (key === "resolution" || key === "framerate") {
      const res = key === "resolution" ? value : config.recording.resolution;
      const fps = key === "framerate" ? value : config.recording.framerate;
      
      let bitrate = "15M"; // Default (1080p60)

      if (res === "native") {
        bitrate = fps === 60 ? "30M" : "15M";
      } else if (res === "1920x1080") {
        bitrate = fps === 60 ? "20M" : "10M"; // Bumped to 20M to match 1440p density
      } else if (res === "1280x720") {
        bitrate = fps === 60 ? "12M" : "6M";  // Bumped to 12M for safety
      }
      
      newConfig.recording.bitrate = bitrate;
      console.log(`Auto-set bitrate to ${bitrate} for ${res} @ ${fps}fps`);
    }

    setConfig(newConfig);
  }

  if (loading) return <div>Loading settings...</div>;
  if (!config) return <div>Error loading settings</div>;

  return (
    <div className="p-6 bg-white rounded-lg shadow-md max-w-2xl mx-auto mt-8 text-left">
      <h2 className="text-2xl font-bold mb-6 text-gray-800">Recording Settings</h2>
      <form onSubmit={handleSave} className="space-y-4">
        
        {/* Audio Source */}
        <div>
          <label className="block text-sm font-medium text-gray-700">Audio Source</label>
          <select
            value={config.recording.audio_source || ""}
            onChange={(e) => updateRecordingConfig("audio_source", e.target.value || null)}
            className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 p-2 border"
          >
            <option value="">None</option>
            {audioDevices.map((device) => (
              <option key={device} value={device}>
                {device}
              </option>
            ))}
          </select>
        </div>

        <div className="grid grid-cols-2 gap-4">
            {/* Resolution */}
            <div>
              <label className="block text-sm font-medium text-gray-700">Resolution</label>
              <select
                value={config.recording.resolution || "native"}
                onChange={(e) => updateRecordingConfig("resolution", e.target.value)}
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 p-2 border"
              >
                <option value="native">Native (Best Performance)</option>
                <option value="1920x1080">1080p (FHD)</option>
                <option value="1280x720">720p (HD)</option>
              </select>
            </div>

            {/* Framerate */}
            <div>
              <label className="block text-sm font-medium text-gray-700">Framerate</label>
              <select
                value={config.recording.framerate}
                onChange={(e) => updateRecordingConfig("framerate", parseInt(e.target.value))}
                className="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 p-2 border"
              >
                <option value={60}>60 FPS (Smooth)</option>
                <option value={30}>30 FPS (Efficient)</option>
              </select>
            </div>
        </div>

        <button
          type="submit"
          disabled={saving}
          className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:bg-gray-400"
        >
          {saving ? "Saving..." : "Save Settings"}
        </button>
      </form>
    </div>
  );
}
