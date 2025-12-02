import { useSettings } from '../hooks/useSettings';
import {
  DEFAULT_BUFFER_SECONDS,
  DEFAULT_SEGMENT_TIME,
  DEFAULT_RESOLUTION,
} from '@squadsync/shared';
import { Tooltip } from './ui/Tooltip';
import {
  Save,
  Monitor,
  Mic,
  Speaker,
  Film,
  LayoutTemplate,
  Settings2,
  Info,
  User,
  Video,
  Volume2,
  Clock,
} from 'lucide-react';

const Section = ({
  title,
  icon: Icon,
  children,
}: {
  title: string;
  icon: any;
  children: React.ReactNode;
}) => (
  <div className="bg-white/5 rounded-2xl p-6 border border-white/5 space-y-4">
    <div className="flex items-center gap-2 text-indigo-400 mb-2">
      <Icon size={18} />
      <h3 className="text-sm font-bold uppercase tracking-wider">{title}</h3>
    </div>
    {children}
  </div>
);

const Select = ({
  label,
  value,
  onChange,
  options,
  icon: Icon,
  tooltip,
}: {
  label: string;
  value: string | number;
  onChange: (val: string) => void;
  options: { label: string; value: string | number }[];
  icon?: any;
  tooltip?: string;
}) => (
  <div className="space-y-2">
    <div className="flex items-center gap-2">
      {Icon && <Icon size={14} className="text-slate-400" />}
      <label className="text-sm font-medium text-slate-300">{label}</label>
      {tooltip && (
        <Tooltip content={tooltip} position="top">
          <Info size={14} className="text-slate-500 hover:text-slate-300 cursor-help" />
        </Tooltip>
      )}
    </div>
    <div className="relative">
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="w-full appearance-none rounded-xl bg-slate-900/50 border border-white/10 text-slate-200 shadow-sm focus:border-indigo-500/50 focus:ring-indigo-500/50 sm:text-sm py-3 pl-4 pr-10 transition-all hover:bg-slate-900/80 hover:border-white/20"
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
      <div className="pointer-events-none absolute inset-y-0 right-0 flex items-center px-4 text-slate-500">
        <svg className="h-4 w-4 fill-current" viewBox="0 0 20 20">
          <path
            fillRule="evenodd"
            d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
            clipRule="evenodd"
          />
        </svg>
      </div>
    </div>
  </div>
);

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
    updateUserConfig,
  } = useSettings();

  async function handleSave(e: React.FormEvent) {
    e.preventDefault();
    await saveSettings();
  }

  if (loading) return <div className="p-8 text-center text-slate-400">Loading settings...</div>;
  if (!config) return <div className="p-8 text-center text-red-400">Error loading settings</div>;

  return (
    <div className="bg-slate-950/80 backdrop-blur-2xl border border-white/10 rounded-3xl shadow-2xl overflow-hidden text-left ring-1 ring-white/5 w-full max-w-2xl mx-auto">
      <div className="p-8 border-b border-white/5 bg-gradient-to-b from-white/5 to-transparent flex items-center justify-between">
        <h2 className="text-xl font-bold text-white flex items-center gap-3">
          <div className="p-2.5 rounded-xl bg-indigo-500/20 text-indigo-400 border border-indigo-500/20 shadow-[0_0_15px_-5px_rgba(99,102,241,0.3)]">
            <Settings2 size={22} />
          </div>
          Settings
        </h2>
      </div>

      <form
        onSubmit={handleSave}
        className="p-8 space-y-8 max-h-[70vh] overflow-y-auto scrollbar-hide"
      >
        {/* User Profile */}
        <Section title="Profile" icon={User}>
          <div className="space-y-2">
            <label className="text-sm font-medium text-slate-300">Display Name</label>
            <input
              type="text"
              value={config.user?.display_name || ''}
              onChange={(e) => updateUserConfig('display_name', e.target.value)}
              placeholder="Enter your name"
              className="w-full rounded-xl bg-slate-900/50 border border-white/10 text-slate-200 shadow-sm focus:border-indigo-500/50 focus:ring-indigo-500/50 sm:text-sm py-3 px-4 transition-all hover:bg-slate-900/80 hover:border-white/20 placeholder:text-slate-600"
            />
          </div>
        </Section>

        {/* Audio */}
        <Section title="Audio" icon={Volume2}>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Select
              label="Microphone"
              icon={Mic}
              value={config.recording.audio_source || ''}
              onChange={(val) => updateRecordingConfig('audio_source', val || null)}
              options={[
                { label: 'None', value: '' },
                ...audioDevices.map((d) => ({ label: d, value: d })),
              ]}
            />
            <Select
              label="System Audio"
              icon={Speaker}
              value={config.recording.system_audio_device || ''}
              onChange={(val) => updateRecordingConfig('system_audio_device', val || null)}
              options={[
                { label: 'None', value: '' },
                ...systemAudioDevices.map((d) => ({ label: d, value: d })),
              ]}
            />
          </div>
        </Section>

        {/* Video */}
        <Section title="Video" icon={Video}>
          <div className="space-y-6">
            <Select
              label="Monitor"
              icon={Monitor}
              value={config.recording.monitor_index || 0}
              onChange={(val) => updateRecordingConfig('monitor_index', parseInt(val))}
              options={monitors.map((m) => ({
                label: `${m.name} (${m.width}x${m.height})${m.is_primary ? ' (Primary)' : ''}`,
                value: m.id,
              }))}
            />
            <div className="grid grid-cols-2 gap-6">
              <Select
                label="Resolution"
                icon={LayoutTemplate}
                value={config.recording.resolution || DEFAULT_RESOLUTION}
                onChange={(val) => updateRecordingConfig('resolution', val)}
                options={[
                  { label: 'Native', value: 'native' },
                  { label: '1080p', value: '1920x1080' },
                  { label: '720p', value: '1280x720' },
                ]}
              />
              <Select
                label="Framerate"
                icon={Film}
                value={config.recording.framerate}
                onChange={(val) => updateRecordingConfig('framerate', parseInt(val))}
                options={[
                  { label: '60 FPS', value: 60 },
                  { label: '30 FPS', value: 30 },
                ]}
              />
            </div>
          </div>
        </Section>

        {/* Buffer */}
        <Section title="Buffer" icon={Clock}>
          <div className="grid grid-cols-2 gap-6">
            <Select
              label="Duration"
              value={config.recording.buffer_duration || DEFAULT_BUFFER_SECONDS}
              onChange={(val) => updateRecordingConfig('buffer_duration', parseInt(val))}
              tooltip="How far back to record"
              options={[
                { label: '30s', value: 30 },
                { label: '1m', value: 60 },
                { label: '2m', value: 120 },
                { label: '3m', value: 180 },
                { label: '4m', value: 240 },
                { label: '5m', value: 300 },
              ]}
            />
            <Select
              label="Segment Size"
              value={config.recording.segment_time || DEFAULT_SEGMENT_TIME}
              onChange={(val) => updateRecordingConfig('segment_time', parseInt(val))}
              tooltip="Smaller is safer (20s recommended)"
              options={[
                { label: '10s', value: 10 },
                { label: '15s', value: 15 },
                { label: '20s', value: 20 },
                { label: '25s', value: 25 },
              ]}
            />
          </div>
        </Section>

        <div className="pt-4">
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
        </div>
      </form>
    </div>
  );
}
