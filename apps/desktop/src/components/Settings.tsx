import { useSettings } from '../hooks/useSettings';
import {
  DEFAULT_BUFFER_SECONDS,
  DEFAULT_SEGMENT_TIME,
  DEFAULT_RESOLUTION,
} from '@squadsync/shared';
import { Tooltip } from './ui/Tooltip';
import { Save, Monitor, Mic, Speaker, Film, LayoutTemplate, Settings2, Info } from 'lucide-react';

export function Settings() {
  const {
    config,
    audioDevices,
    systemAudioDevices,
    monitors,
    loading,
    saving,
    saveSettings,
    updateRecordingConfig,
  } = useSettings();

  async function handleSave(e: React.FormEvent) {
    e.preventDefault();
    await saveSettings();
  }

  if (loading) return <div className="p-8 text-center text-slate-400">Loading settings...</div>;
  if (!config) return <div className="p-8 text-center text-red-400">Error loading settings</div>;

  return (
    <div className="bg-slate-900/40 backdrop-blur-2xl border border-white/10 rounded-3xl shadow-2xl overflow-hidden text-left ring-1 ring-white/5 w-full max-w-lg">
      <div className="p-8 border-b border-white/5 bg-gradient-to-b from-white/5 to-transparent">
        <h2 className="text-lg font-bold text-white flex items-center gap-3">
          <div className="p-2 rounded-lg bg-indigo-500/20 text-indigo-400">
            <Settings2 size={20} />
          </div>
          Recording Settings
        </h2>
      </div>

      <form onSubmit={handleSave} className="p-8 space-y-8">
        {/* Audio Section */}
        <div className="space-y-6">
          <h3 className="text-xs font-bold text-slate-500 uppercase tracking-wider mb-3">
            Audio Sources
          </h3>

          <div className="space-y-1">
            <label className="text-sm font-medium text-slate-300 flex items-center gap-2">
              <Mic size={16} className="text-slate-400" /> Microphone
            </label>
            <select
              value={config.recording.audio_source || ''}
              onChange={(e) => updateRecordingConfig('audio_source', e.target.value || null)}
              className="w-full rounded-xl bg-slate-900/50 border-white/10 text-slate-200 shadow-sm focus:border-indigo-500/50 focus:ring-indigo-500/50 sm:text-sm py-3 transition-colors hover:bg-slate-900/70"
            >
              <option value="">None</option>
              {audioDevices.map((device) => (
                <option key={device} value={device}>
                  {device}
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-1">
            <label className="text-sm font-medium text-slate-300 flex items-center gap-2">
              <Speaker size={16} className="text-slate-400" /> System Audio
            </label>
            <select
              value={config.recording.system_audio_device || ''}
              onChange={(e) => updateRecordingConfig('system_audio_device', e.target.value || null)}
              className="w-full rounded-xl bg-slate-900/50 border-white/10 text-slate-200 shadow-sm focus:border-indigo-500/50 focus:ring-indigo-500/50 sm:text-sm py-3 transition-colors hover:bg-slate-900/70"
            >
              <option value="">None</option>
              {systemAudioDevices.map((device) => (
                <option key={device} value={device}>
                  {device}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="h-px bg-gradient-to-r from-transparent via-white/10 to-transparent" />

        {/* Video Section */}
        <div className="space-y-6">
          <h3 className="text-xs font-bold text-slate-500 uppercase tracking-wider mb-3">
            Video & Display
          </h3>

          <div className="space-y-1">
            <label className="text-sm font-medium text-slate-300 flex items-center gap-2">
              <Monitor size={16} className="text-slate-400" /> Monitor
            </label>
            <select
              value={config.recording.monitor_index || 0}
              onChange={(e) => updateRecordingConfig('monitor_index', parseInt(e.target.value))}
              className="w-full rounded-xl bg-slate-900/50 border-white/10 text-slate-200 shadow-sm focus:border-indigo-500/50 focus:ring-indigo-500/50 sm:text-sm py-3 transition-colors hover:bg-slate-900/70"
            >
              {monitors.map((monitor) => (
                <option key={monitor.id} value={monitor.id}>
                  {monitor.name} ({monitor.width}x{monitor.height}){' '}
                  {monitor.is_primary ? '(Primary)' : ''}
                </option>
              ))}
            </select>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1">
              <label className="text-sm font-medium text-slate-300 flex items-center gap-2">
                <LayoutTemplate size={16} className="text-slate-400" /> Resolution
              </label>
              <select
                value={config.recording.resolution || DEFAULT_RESOLUTION}
                onChange={(e) => updateRecordingConfig('resolution', e.target.value)}
                className="w-full rounded-xl bg-slate-900/50 border-white/10 text-slate-200 shadow-sm focus:border-indigo-500/50 focus:ring-indigo-500/50 sm:text-sm py-3 transition-colors hover:bg-slate-900/70"
              >
                <option value="native">Native</option>
                <option value="1920x1080">1080p</option>
                <option value="1280x720">720p</option>
              </select>
            </div>

            <div className="space-y-1">
              <label className="text-sm font-medium text-slate-300 flex items-center gap-2">
                <Film size={16} className="text-slate-400" /> Framerate
              </label>
              <select
                value={config.recording.framerate}
                onChange={(e) => updateRecordingConfig('framerate', parseInt(e.target.value))}
                className="w-full rounded-xl bg-slate-900/50 border-white/10 text-slate-200 shadow-sm focus:border-indigo-500/50 focus:ring-indigo-500/50 sm:text-sm py-3 transition-colors hover:bg-slate-900/70"
              >
                <option value={60}>60 FPS</option>
                <option value={30}>30 FPS</option>
              </select>
            </div>
          </div>
        </div>

        <div className="h-px bg-gradient-to-r from-transparent via-white/10 to-transparent" />

        {/* Buffer Section */}
        <div className="space-y-6">
          <h3 className="text-xs font-bold text-slate-500 uppercase tracking-wider mb-3">
            Replay Buffer
          </h3>
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1">
              <div className="flex items-center gap-2">
                <label className="block text-sm font-medium text-slate-300">Duration</label>
                <Tooltip content="How far back the replay buffer will record." position="top">
                  <Info size={14} className="text-slate-500 hover:text-slate-300 cursor-help" />
                </Tooltip>
              </div>
              <select
                value={config.recording.buffer_duration || DEFAULT_BUFFER_SECONDS}
                onChange={(e) => updateRecordingConfig('buffer_duration', parseInt(e.target.value))}
                className="w-full rounded-xl bg-slate-900/50 border-white/10 text-slate-200 shadow-sm focus:border-indigo-500/50 focus:ring-indigo-500/50 sm:text-sm py-3 transition-colors hover:bg-slate-900/70"
              >
                <option value={30}>30s</option>
                <option value={60}>1m</option>
                <option value={120}>2m</option>
                <option value={180}>3m</option>
                <option value={240}>4m</option>
                <option value={300}>5m</option>
              </select>
            </div>

            <div className="space-y-1">
              <div className="flex items-center gap-2">
                <label className="block text-sm font-medium text-slate-300">Segment Size</label>
                <Tooltip
                  content="Smaller segments are safer but create more files. 20s is recommended."
                  position="top"
                >
                  <Info size={14} className="text-slate-500 hover:text-slate-300 cursor-help" />
                </Tooltip>
              </div>
              <select
                value={config.recording.segment_time || DEFAULT_SEGMENT_TIME}
                onChange={(e) => updateRecordingConfig('segment_time', parseInt(e.target.value))}
                className="w-full rounded-xl bg-slate-900/50 border-white/10 text-slate-200 shadow-sm focus:border-indigo-500/50 focus:ring-indigo-500/50 sm:text-sm py-3 transition-colors hover:bg-slate-900/70"
              >
                <option value={10}>10s</option>
                <option value={15}>15s</option>
                <option value={20}>20s</option>
                <option value={25}>25s</option>
              </select>
            </div>
          </div>
        </div>

        <button
          type="submit"
          disabled={saving}
          className="w-full flex justify-center items-center gap-2 py-4 px-4 border border-indigo-400/20 rounded-xl shadow-[0_0_20px_-5px_rgba(79,70,229,0.3)] text-sm font-bold text-white bg-gradient-to-r from-indigo-600 to-violet-600 hover:from-indigo-500 hover:to-violet-500 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-slate-900 focus:ring-indigo-500 disabled:bg-slate-700 disabled:text-slate-500 transition-all hover:scale-[1.02] active:scale-[0.98]"
        >
          {saving ? (
            <>Saving...</>
          ) : (
            <>
              <Save size={18} /> Save Changes
            </>
          )}
        </button>
      </form>
    </div>
  );
}
