'use client';

import { useRef, useState, useEffect, useCallback, useMemo } from 'react';
import { Play, Pause, Volume2, VolumeX, Loader2 } from 'lucide-react';
import { View, computeTimelineStartMs, computeTimelineEndMs } from '@squadsync/shared';
import { VideoTile } from './VideoTile';

interface WebSquadGridProps {
  clips: View[];
}

interface SyncStat {
  author: string;
  drift: string;
  rate: string;
  target: string;
  current: string;
}

export function WebSquadGrid({ clips }: WebSquadGridProps) {
  const [isPlaying, setIsPlaying] = useState(false);
  const [globalCurrentTimeMs, setGlobalCurrentTimeMs] = useState(0);
  const [muted, setMuted] = useState(true);
  const [isScrubbing, setIsScrubbing] = useState(false);
  const [isBuffering, setIsBuffering] = useState(false);
  const [videosReady, setVideosReady] = useState(false);

  // 1. Timeline Metrics
  const timelineStartMs = useMemo(() => {
    return computeTimelineStartMs(clips);
  }, [clips]);

  const timelineEndMs = useMemo(() => {
    return computeTimelineEndMs(clips);
  }, [clips]);

  const globalDurationMs = useMemo(() => {
    if (!timelineStartMs || !timelineEndMs || timelineEndMs <= timelineStartMs) return 1000;
    return timelineEndMs - timelineStartMs;
  }, [timelineStartMs, timelineEndMs]);

  // Refs
  const videoRefs = useRef<Map<string, HTMLVideoElement>>(new Map());
  const requestRef = useRef<number | undefined>(undefined);
  const previousTimeRef = useRef<number | undefined>(undefined);
  const animateRef = useRef<((time: number) => void) | undefined>(undefined);

  // Buffering State Tracker (Ref for sync loop access)
  const bufferingStateRef = useRef<Map<string, boolean>>(new Map());

  // Debug
  const debugRef = useRef<HTMLDivElement>(null);
  const [showDebug, setShowDebug] = useState(false);

  // Optimization: Cached list of active views to avoid Map.get() in loop
  const activeViews = useMemo(() => clips, [clips]);

  // 2. Core Sync Logic
  const syncVideos = useCallback(
    (globalTime: number, forceSeek: boolean = false) => {
      if (!timelineStartMs) return;

      const stats: SyncStat[] = [];

      activeViews.forEach((clip) => {
        const video = videoRefs.current.get(clip.url);
        if (!video) return;

        // Calculate target time in this specific video file
        // target = (globalTime - (clipStart - timelineStart)) / 1000
        // Simplifies to: (globalTime + timelineStart - clipStart) / 1000
        const targetVideoTimeSec = (timelineStartMs + globalTime - clip.videoStartTimeMs) / 1000;

        // Clip hasn't started yet or has ended in the global timeline
        if (targetVideoTimeSec < 0 || targetVideoTimeSec > video.duration) {
          if (!video.paused) video.pause();
          // We don't correct drift for inactive videos
          return;
        }

        // Ensure active videos are playing if we are playing (and not buffering globally)
        if (isPlaying && !isBuffering && video.paused && !forceSeek) {
          video.play().catch(() => {});
        } else if (!isPlaying && !video.paused) {
          video.pause();
        }

        const diff = video.currentTime - targetVideoTimeSec;
        const absDiff = Math.abs(diff);

        stats.push({
          author: clip.author,
          drift: diff.toFixed(3),
          rate: video.playbackRate.toFixed(2),
          target: targetVideoTimeSec.toFixed(2),
          current: video.currentTime.toFixed(2),
        });

        // --- Drift Correction Strategy (Proportional Control) ---
        const MAX_DRIFT = 0.5; // Force seek if drift > 500ms
        const SYNC_THRESHOLD = 0.015; // Synced if drift < 15ms
        const P_GAIN = 0.15; // Proportional Gain: Rate change per second of drift
        // e.g. 100ms drift * 0.15 = 0.015 rate change (1.5% speedup)

        // 1. Force Seek (User scrub or massive drift)
        if (forceSeek || absDiff > MAX_DRIFT) {
          video.currentTime = targetVideoTimeSec;
          video.playbackRate = 1.0;
        }
        // 2. Proportional Rate Control
        else if (absDiff > SYNC_THRESHOLD) {
          // Calculate desired correction
          // If diff is positive (video is ahead), we want rate < 1.0
          // If diff is negative (video is behind), we want rate > 1.0
          // diff = video.currentTime - target
          // Using: rate = 1.0 - (diff * P_GAIN)

          let targetRate = 1.0 - diff * P_GAIN;

          // Clamp rate to avoid audio distortion
          // 0.95 to 1.05 is usually safe for pitch-corrected audio
          targetRate = Math.max(0.95, Math.min(1.05, targetRate));

          video.playbackRate = targetRate;
        }
        // 3. Synced
        else {
          if (video.playbackRate !== 1.0) {
            video.playbackRate = 1.0;
          }
        }
      });

      if (debugRef.current && showDebug) {
        debugRef.current.innerText =
          `GlobalTime: ${globalTime.toFixed(1)} ${isBuffering ? '(BUFFERING)' : ''}\n` +
          stats.map((s) => `${s.author}: Drift=${s.drift}s Rate=${s.rate}x`).join('\n');
      }
    },
    [timelineStartMs, isPlaying, isBuffering, showDebug, activeViews]
  );

  // 3. Pre-Roll / Initial Seek
  const lastTimelineStartMsRef = useRef<number | null>(null);

  // When clips change, we want to snap everything to 0:00 (Global) immediately
  useEffect(() => {
    if (activeViews.length > 0 && timelineStartMs > 0) {
      // Only reset/force seek if the timeline start has changed (new session or shift)
      if (lastTimelineStartMsRef.current !== timelineStartMs) {
        lastTimelineStartMsRef.current = timelineStartMs;

        // Wait a tick for video elements to mount
        const timer = setTimeout(() => {
          syncVideos(0, true); // Force seek to 0
          setVideosReady(true);
        }, 100);
        return () => clearTimeout(timer);
      }
    }
  }, [activeViews, timelineStartMs, syncVideos]);

  // 4. Global Buffering Check
  const checkGlobalBuffering = useCallback(() => {
    let anyBuffering = false;

    activeViews.forEach((clip) => {
      const isBuf = bufferingStateRef.current.get(clip.url);
      // Only care if video SHOULD be playing (within its time range)
      if (isBuf) {
        const video = videoRefs.current.get(clip.url);
        // If video is active in current timeframe
        if (video && !video.paused) {
          anyBuffering = true;
        }
      }
    });

    // Update state only if changed to avoid re-renders
    setIsBuffering((prev) => {
      if (prev !== anyBuffering) return anyBuffering;
      return prev;
    });

    return anyBuffering;
  }, [activeViews]);

  // 5. The Sync Loop
  const animate = useCallback(
    (time: number) => {
      if (previousTimeRef.current !== undefined) {
        const deltaTime = time - previousTimeRef.current;

        if (isPlaying && !isScrubbing) {
          // Check buffering
          const currentlyBuffering = checkGlobalBuffering();

          if (!currentlyBuffering) {
            setGlobalCurrentTimeMs((prev) => {
              const newTime = prev + deltaTime;
              if (newTime >= globalDurationMs) {
                setIsPlaying(false);
                syncVideos(globalDurationMs, true);
                return globalDurationMs;
              }
              syncVideos(newTime);
              return newTime;
            });
          } else {
            // If buffering, we don't advance time, but we might mistakenly fall out of sync?
            // Ideally we pause the "good" videos?
            // syncVideos() is called with CURRENT time, which will pause videos if they are ahead.
            // But we need to ensure we call it to enforce the pause.
            syncVideos(globalCurrentTimeMs);
          }
        }
      }
      previousTimeRef.current = time;
      if (isPlaying) {
        requestRef.current = requestAnimationFrame((t) => animateRef.current?.(t));
      }
    },
    [
      isPlaying,
      isScrubbing,
      globalDurationMs,
      syncVideos,
      checkGlobalBuffering,
      globalCurrentTimeMs,
    ]
  );

  useEffect(() => {
    animateRef.current = animate;
  }, [animate]);

  useEffect(() => {
    if (isPlaying) {
      requestRef.current = requestAnimationFrame((t) => animateRef.current?.(t));
    }
    return () => {
      if (requestRef.current) cancelAnimationFrame(requestRef.current);
    };
  }, [isPlaying]);

  // 6. Stable Handlers for VideoTile
  const handleVideoMount = useCallback((url: string, el: HTMLVideoElement) => {
    videoRefs.current.set(url, el);
  }, []);

  const handleVideoUnmount = useCallback((url: string) => {
    videoRefs.current.delete(url);
    bufferingStateRef.current.delete(url);
  }, []);

  const handleVideoWaiting = useCallback((url: string) => {
    bufferingStateRef.current.set(url, true);
  }, []);

  const handleVideoPlaying = useCallback((url: string) => {
    bufferingStateRef.current.set(url, false);
  }, []);

  // Handlers
  const handlePlay = () => {
    // Resume audio context if needed (browser policy)
    setIsPlaying(true);
    previousTimeRef.current = performance.now();
  };

  const handlePause = () => {
    setIsPlaying(false);
    videoRefs.current.forEach((v) => v.pause());
  };

  const handleSeek = (timeMs: number) => {
    setGlobalCurrentTimeMs(timeMs);
    syncVideos(timeMs, true); // Force seek
  };

  // Robust Scrubber Release
  useEffect(() => {
    const handleUp = () => {
      if (isScrubbing) {
        setIsScrubbing(false);
        if (isPlaying) previousTimeRef.current = performance.now();
      }
    };
    if (isScrubbing) {
      window.addEventListener('mouseup', handleUp);
      window.addEventListener('touchend', handleUp);
    }
    return () => {
      window.removeEventListener('mouseup', handleUp);
      window.removeEventListener('touchend', handleUp);
    };
  }, [isScrubbing, isPlaying]);

  const getGridClass = (count: number) => {
    if (count <= 1) return 'grid-cols-1';
    if (count <= 4) return 'grid-cols-2';
    if (count <= 9) return 'grid-cols-3';
    return 'grid-cols-4';
  };

  return (
    <div className="flex flex-col h-full bg-slate-950 relative">
      {/* Visual Debug Overlay */}
      <div
        ref={debugRef}
        className={`absolute top-4 right-4 z-50 bg-black/80 text-green-400 font-mono text-[10px] p-2 rounded pointer-events-none whitespace-pre ${showDebug ? 'block' : 'hidden'}`}
      />

      {/* Grid */}
      <div
        className={`flex-1 grid ${getGridClass(clips.length)} gap-4 p-4 overflow-hidden relative`}
      >
        {/* Global Loading Spinner */}
        {(isBuffering || !videosReady) && (
          <div className="absolute inset-0 z-40 bg-black/20 backdrop-blur-sm flex items-center justify-center pointer-events-none">
            <Loader2 size={48} className="text-white animate-spin" />
          </div>
        )}

        {clips.map((clip) => (
          <VideoTile
            key={clip.url}
            clip={clip}
            muted={muted}
            onMount={handleVideoMount}
            onUnmount={handleVideoUnmount}
            onWaiting={handleVideoWaiting}
            onPlaying={handleVideoPlaying}
          />
        ))}
      </div>

      {/* Controls */}
      <div className="h-20 border-t border-white/10 bg-slate-900/50 backdrop-blur-xl px-6 flex items-center gap-6">
        <button
          onClick={isPlaying ? handlePause : handlePlay}
          className="w-12 h-12 rounded-full bg-indigo-600 hover:bg-indigo-500 flex items-center justify-center text-white transition-all hover:scale-105"
        >
          {isPlaying ? (
            <Pause size={20} className="fill-current" />
          ) : (
            <Play size={20} className="fill-current ml-1" />
          )}
        </button>

        {/* Scrubber */}
        <div className="flex-1 flex flex-col gap-2">
          <input
            type="range"
            min={0}
            max={globalDurationMs}
            value={globalCurrentTimeMs}
            onChange={(e) => handleSeek(parseFloat(e.target.value))}
            className="w-full cursor-pointer accent-indigo-500"
            onMouseDown={() => setIsScrubbing(true)}
            onTouchStart={() => setIsScrubbing(true)}
          />
          <div className="flex justify-between text-xs text-slate-400 font-mono">
            <span>{formatTime(globalCurrentTimeMs / 1000)}</span>
            <span>{formatTime(globalDurationMs / 1000)}</span>
          </div>
        </div>

        <button onClick={() => setMuted(!muted)} className="text-slate-400 hover:text-white">
          {muted ? <VolumeX size={20} /> : <Volume2 size={20} />}
        </button>

        {/* Debug Toggle */}
        <button
          onClick={() => setShowDebug(!showDebug)}
          className={`text-[10px] uppercase font-bold tracking-wider px-2 py-1 rounded border ${showDebug ? 'bg-indigo-500/20 text-indigo-400 border-indigo-500/30' : 'text-slate-600 border-slate-700 hover:text-slate-400'}`}
        >
          Debug
        </button>
      </div>
    </div>
  );
}

function formatTime(seconds: number) {
  const m = Math.floor(seconds / 60);
  const s = Math.floor(seconds % 60);
  return `${m}:${s.toString().padStart(2, '0')}`;
}
