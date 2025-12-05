'use client';

import { useRef, useState, useEffect, useCallback, useMemo } from 'react';
import { Play, Pause, Volume2, VolumeX } from 'lucide-react';
import {
  View,
  computeTimelineStartMs,
  computeTimelineEndMs,
  computeClipOffsetMs,
} from '@squadsync/shared';

interface WebSquadGridProps {
  clips: View[];
}

export function WebSquadGrid({ clips }: WebSquadGridProps) {
  const [isPlaying, setIsPlaying] = useState(false);
  const [globalCurrentTimeMs, setGlobalCurrentTimeMs] = useState(0);
  const [muted, setMuted] = useState(true);
  const [isScrubbing, setIsScrubbing] = useState(false);

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

  const videoRefs = useRef<Map<string, HTMLVideoElement>>(new Map());
  const requestRef = useRef<number | undefined>(undefined);
  const previousTimeRef = useRef<number | undefined>(undefined);
  const animateRef = useRef<((time: number) => void) | undefined>(undefined);

  const debugRef = useRef<HTMLDivElement>(null);
  const [showDebug, setShowDebug] = useState(false);

  // 2. Drift Correction Logic
  const syncVideos = useCallback(
    (globalTime: number) => {
      if (!timelineStartMs) return;

      const stats: any[] = [];

      clips.forEach((clip) => {
        const video = videoRefs.current.get(clip.url);
        if (!video) return;

        // Calculate target time in the video file
        const targetVideoTimeSec = (timelineStartMs + globalTime - clip.videoStartTimeMs) / 1000;
        const diff = video.currentTime - targetVideoTimeSec;

        stats.push({
          author: clip.author,
          drift: diff.toFixed(3),
          rate: video.playbackRate.toFixed(2),
          target: targetVideoTimeSec.toFixed(2),
          current: video.currentTime.toFixed(2),
        });

        // If video hasn't started yet or has ended
        if (targetVideoTimeSec < 0 || targetVideoTimeSec > video.duration) {
          if (!video.paused) video.pause();
          return;
        }

        if (video.paused && isPlaying) {
          video.play().catch(() => {});
        }

        // Hard Seek (Drift > 0.2s)
        if (Math.abs(diff) > 0.2) {
          video.currentTime = targetVideoTimeSec;
        }
        // Micro-Adjustment (Drift > 0.05s)
        else if (Math.abs(diff) > 0.05) {
          video.playbackRate = diff > 0 ? 0.95 : 1.05;
        }
        // In Sync
        else {
          video.playbackRate = 1.0;
        }
      });

      // Valid debug UI update without console spam
      if (debugRef.current && showDebug) {
        debugRef.current.innerText =
          `GlobalTime: ${globalTime.toFixed(1)}\n` +
          stats.map((s) => `${s.author}: Drift=${s.drift}s Rate=${s.rate}x`).join('\n');
      }
    },
    [clips, timelineStartMs, isPlaying, showDebug]
  );

  // 3. The Sync Loop
  const animate = useCallback(
    (time: number) => {
      if (previousTimeRef.current !== undefined) {
        const deltaTime = time - previousTimeRef.current;

        if (isPlaying && !isScrubbing) {
          setGlobalCurrentTimeMs((prev) => {
            const newTime = prev + deltaTime;
            if (newTime >= globalDurationMs) {
              setIsPlaying(false);
              syncVideos(globalDurationMs);
              return globalDurationMs;
            }
            syncVideos(newTime);
            return newTime;
          });
        }
      }
      previousTimeRef.current = time;
      if (isPlaying) {
        requestRef.current = requestAnimationFrame((t) => animateRef.current?.(t));
      }
    },
    [isPlaying, syncVideos, isScrubbing, globalDurationMs]
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

  const handlePlay = () => {
    setIsPlaying(true);
    previousTimeRef.current = performance.now();
  };

  const handlePause = () => {
    setIsPlaying(false);
    videoRefs.current.forEach((v) => v.pause());
  };

  const handleSeek = (timeMs: number) => {
    setGlobalCurrentTimeMs(timeMs);
    syncVideos(timeMs);
    syncVideos(timeMs); // Double sync to ensure seek update
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
      <div className={`flex-1 grid ${getGridClass(clips.length)} gap-4 p-4 overflow-hidden`}>
        {clips.map((clip) => (
          // ... (video rendering same)
          <div
            key={clip.url}
            className="relative group rounded-xl overflow-hidden bg-slate-900 border border-white/10"
          >
            <video
              ref={(el) => {
                if (el) videoRefs.current.set(clip.url, el);
                else videoRefs.current.delete(clip.url);
              }}
              src={clip.url}
              className="w-full h-full object-contain"
              muted={muted}
              playsInline
            />
            <div className="absolute bottom-4 left-4 px-3 py-1 rounded-full bg-black/60 backdrop-blur-md text-white text-xs font-medium">
              {clip.author}
            </div>
          </div>
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
