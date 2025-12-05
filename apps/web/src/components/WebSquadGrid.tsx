'use client';

import { useRef, useState, useEffect, useCallback, useMemo } from 'react';
import { Play, Pause, Volume2, VolumeX } from 'lucide-react';
import { View, computeTimelineStartMs, computeClipOffsetMs } from '@squadsync/shared';

interface WebSquadGridProps {
  clips: View[];
}

export function WebSquadGrid({ clips }: WebSquadGridProps) {
  const [isPlaying, setIsPlaying] = useState(false);
  const [globalCurrentTimeMs, setGlobalCurrentTimeMs] = useState(0);
  const [globalDurationMs, setGlobalDurationMs] = useState(60 * 1000); // Default 60s
  const [muted, setMuted] = useState(true);

  const timelineStartMs = useMemo(() => {
    return computeTimelineStartMs(clips);
  }, [clips]);

  const videoRefs = useRef<Map<string, HTMLVideoElement>>(new Map());
  const requestRef = useRef<number | undefined>(undefined);
  const previousTimeRef = useRef<number | undefined>(undefined);
  const animateRef = useRef<((time: number) => void) | undefined>(undefined);

  // 2. Drift Correction Logic
  const syncVideos = useCallback(
    (globalTime: number) => {
      if (!timelineStartMs) return;

      clips.forEach((clip) => {
        const video = videoRefs.current.get(clip.url); // Using URL as ID for now
        if (!video) return;

        const offset = computeClipOffsetMs(clip, timelineStartMs);
        const targetVideoTimeSec = (globalTime - offset) / 1000;

        // If video hasn't started yet or has ended
        if (targetVideoTimeSec < 0 || targetVideoTimeSec > video.duration) {
          if (!video.paused) video.pause();
          return;
        }

        // If video should be playing but is paused
        if (video.paused && isPlaying) {
          video.play().catch(() => {});
        }

        const diff = video.currentTime - targetVideoTimeSec;

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
    },
    [clips, timelineStartMs, isPlaying]
  );

  // 3. The Sync Loop
  const animate = useCallback(
    (time: number) => {
      if (previousTimeRef.current !== undefined) {
        const deltaTime = time - previousTimeRef.current;

        if (isPlaying) {
          setGlobalCurrentTimeMs((prev) => {
            const newTime = prev + deltaTime;
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
    [isPlaying, syncVideos]
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
  };

  const getGridClass = (count: number) => {
    if (count <= 1) return 'grid-cols-1';
    if (count <= 4) return 'grid-cols-2';
    if (count <= 9) return 'grid-cols-3';
    return 'grid-cols-4';
  };

  return (
    <div className="flex flex-col h-full bg-slate-950">
      {/* Grid */}
      <div className={`flex-1 grid ${getGridClass(clips.length)} gap-4 p-4 overflow-hidden`}>
        {clips.map((clip) => (
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
              // Disable native controls, we control it programmatically
            />
            <div className="absolute bottom-4 left-4 px-3 py-1 rounded-full bg-black/60 backdrop-blur-md text-white text-xs font-medium">
              {clip.author}
            </div>
            {/* Debug Info */}
            <div className="absolute top-4 left-4 px-2 py-1 rounded bg-black/40 text-[10px] text-white font-mono opacity-0 group-hover:opacity-100 transition-opacity">
              Start: {new Date(clip.videoStartTimeMs).toLocaleTimeString()}
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
            className="w-full h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer accent-indigo-500"
          />
          <div className="flex justify-between text-xs text-slate-400 font-mono">
            <span>{formatTime(globalCurrentTimeMs / 1000)}</span>
            <span>{formatTime(globalDurationMs / 1000)}</span>
          </div>
        </div>

        <button onClick={() => setMuted(!muted)} className="text-slate-400 hover:text-white">
          {muted ? <VolumeX size={20} /> : <Volume2 size={20} />}
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
