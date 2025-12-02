import { useState } from 'react';
import { Settings } from './components/Settings';
import { Tooltip } from './components/ui/Tooltip';
import { RoomManager } from './components/room/RoomManager';
import { Settings2, Circle, Disc, Square, CheckCircle2, X, Info, Film } from 'lucide-react';
import { LocalPlaybackView } from './components/playback/LocalPlaybackView';
import { useToastStore } from './stores/toastStore';
import { useRecorder } from './hooks/useRecorder';

function App() {
  const { toast } = useToastStore();
  const { status, isReplayActive, isBuffering, enableReplay, disableReplay, saveReplay } =
    useRecorder();

  const [showSettings, setShowSettings] = useState(false);
  const [showRecordings, setShowRecordings] = useState(false);

  return (
    <>
      <div className="w-full max-w-lg mx-auto relative z-10">
        {/* Main Card */}
        <div className="bg-slate-900/40 backdrop-blur-2xl border border-white/10 rounded-3xl shadow-[0_0_40px_-10px_rgba(0,0,0,0.5)] ring-1 ring-white/5">
          {/* Header */}
          <div className="p-8 border-b border-white/5 flex justify-between items-center bg-gradient-to-b from-white/5 to-transparent rounded-t-3xl">
            <div className="flex items-center gap-4">
              <div className="w-12 h-12 rounded-2xl bg-indigo-500/20 flex items-center justify-center border border-indigo-500/30 text-indigo-400 shadow-[0_0_15px_-3px_rgba(99,102,241,0.3)]">
                <Disc size={24} />
              </div>
              <div>
                <h1 className="text-xl font-bold text-white tracking-tight">SquadSync</h1>
                <div className="flex items-center gap-2 text-xs font-medium mt-0.5">
                  <span
                    className={`w-2 h-2 rounded-full transition-all duration-500 ${isReplayActive ? 'bg-emerald-500 shadow-[0_0_10px_rgba(16,185,129,0.6)] animate-pulse-soft' : isBuffering ? 'bg-amber-500 animate-pulse' : 'bg-slate-600'}`}
                  ></span>
                  <span
                    className={`transition-colors duration-300 ${isReplayActive ? 'text-emerald-400' : 'text-slate-500'}`}
                  >
                    {status}
                  </span>
                </div>
              </div>
            </div>

            <Tooltip content="Recordings" position="bottom">
              <button
                onClick={() => setShowRecordings(!showRecordings)}
                className={`p-2.5 rounded-xl transition-all duration-200 ${showRecordings ? 'bg-indigo-500/20 text-indigo-400' : 'text-slate-400 hover:text-white hover:bg-white/5'}`}
              >
                <Film size={20} />
              </button>
            </Tooltip>

            <Tooltip content="Settings" position="right">
              <button
                onClick={() => setShowSettings(!showSettings)}
                className={`p-2.5 rounded-xl transition-all duration-200 ${showSettings ? 'bg-indigo-500/20 text-indigo-400' : 'text-slate-400 hover:text-white hover:bg-white/5'}`}
              >
                <Settings2 size={20} />
              </button>
            </Tooltip>
          </div>

          {/* Content */}
          <div className="p-10 flex flex-col items-center gap-8">
            {!isReplayActive ? (
              <button
                type="button"
                className={`group relative w-full py-7 px-8 rounded-2xl font-bold text-lg transition-all duration-300 overflow-hidden ${
                  isBuffering
                    ? 'bg-slate-800/50 text-slate-500 cursor-wait border border-white/5'
                    : 'bg-gradient-to-r from-indigo-600 to-violet-600 hover:from-indigo-500 hover:to-violet-500 text-white shadow-[0_0_30px_-5px_rgba(79,70,229,0.3)] border border-indigo-400/20 hover:scale-[1.02] hover:shadow-[0_0_40px_-5px_rgba(79,70,229,0.4)]'
                }`}
                onClick={enableReplay}
                disabled={isBuffering}
              >
                <div className="flex items-center justify-center gap-3">
                  {isBuffering ? (
                    <>
                      <div className="w-5 h-5 border-2 border-slate-500 border-t-transparent rounded-full animate-spin"></div>
                      <span>Initializing...</span>
                    </>
                  ) : (
                    <>
                      <Circle
                        size={20}
                        className="fill-current opacity-50 group-hover:scale-110 transition-transform duration-300"
                      />
                      <span>Enable Replay Buffer</span>
                    </>
                  )}
                </div>
              </button>
            ) : (
              <div className="flex flex-col w-full gap-4">
                <Tooltip
                  content="Save the last few minutes (Alt+F10)"
                  position="top"
                  className="w-full"
                >
                  <button
                    type="button"
                    className="group w-full py-9 rounded-2xl font-bold text-2xl transition-all duration-300 bg-gradient-to-r from-emerald-600 to-teal-600 hover:from-emerald-500 hover:to-teal-500 text-white shadow-[0_0_30px_-5px_rgba(16,185,129,0.3)] border border-emerald-400/20 hover:scale-[1.02] active:scale-[0.98] hover:shadow-[0_0_40px_-5px_rgba(16,185,129,0.4)]"
                    onClick={() => saveReplay()}
                  >
                    <div className="flex flex-col items-center gap-2">
                      <div className="flex items-center gap-3">
                        <CheckCircle2
                          size={28}
                          className="fill-emerald-800/20 group-hover:scale-110 transition-transform duration-300"
                        />
                        <span>Save Clip</span>
                      </div>
                      <span className="text-xs font-medium text-emerald-100/60 uppercase tracking-widest">
                        Instant Replay
                      </span>
                    </div>
                  </button>
                </Tooltip>

                <button
                  type="button"
                  className="w-full py-4 rounded-xl font-medium text-slate-500 hover:text-red-400 hover:bg-red-500/10 transition-all duration-200 flex items-center justify-center gap-2 text-sm border border-transparent hover:border-red-500/10"
                  onClick={disableReplay}
                >
                  <Square size={14} className="fill-current" />
                  Stop Buffer
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Toast Notification */}
      {toast && (
        <div className="fixed bottom-6 left-1/2 -translate-x-1/2 z-[100] animate-in slide-in-from-bottom-5 fade-in duration-300">
          <div
            className={`px-5 py-3 rounded-2xl shadow-2xl border backdrop-blur-md flex items-center gap-3 ${
              toast.type === 'success'
                ? 'bg-emerald-900/80 border-emerald-500/30 text-emerald-100 shadow-emerald-900/20'
                : 'bg-red-900/80 border-red-500/30 text-red-100 shadow-red-900/20'
            }`}
          >
            {toast.type === 'success' ? <CheckCircle2 size={18} /> : <Info size={18} />}
            <span className="text-sm font-medium">{toast.message}</span>
          </div>
        </div>
      )}

      {/* Settings Modal Overlay */}
      {showSettings && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/60 backdrop-blur-sm p-4 animate-in fade-in duration-200"
          onClick={() => setShowSettings(false)}
        >
          <div
            className="w-full max-w-lg animate-in zoom-in-95 slide-in-from-bottom-2 duration-200 max-h-[90vh] overflow-y-auto rounded-3xl scrollbar-hide shadow-2xl"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="relative">
              <Settings />
              <button
                onClick={() => setShowSettings(false)}
                className="absolute top-4 right-4 p-2 text-slate-400 hover:text-white bg-slate-800/50 hover:bg-slate-700 rounded-full transition-colors z-10"
              >
                <span className="sr-only">Close</span>
                <X size={20} />
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Recordings Modal Overlay */}
      {showRecordings && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/60 backdrop-blur-sm p-4 animate-in fade-in duration-200"
          onClick={() => setShowRecordings(false)}
        >
          <div
            className="w-full max-w-5xl h-[80vh] animate-in zoom-in-95 slide-in-from-bottom-2 duration-200 bg-slate-900/90 backdrop-blur-xl border border-white/10 rounded-3xl shadow-2xl overflow-hidden flex flex-col"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="relative flex-1 p-6 overflow-hidden">
              <LocalPlaybackView />
              <button
                onClick={() => setShowRecordings(false)}
                className="absolute top-6 right-6 p-2 text-slate-400 hover:text-white bg-slate-800/50 hover:bg-slate-700 rounded-full transition-colors z-10"
              >
                <span className="sr-only">Close</span>
                <X size={20} />
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Room Manager Overlay */}
      <div className="fixed bottom-4 right-4 z-40">
        <RoomManager onClipStart={saveReplay} />
      </div>
    </>
  );
}

export default App;
