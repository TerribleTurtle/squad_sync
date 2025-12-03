'use client';

import { useRef, useState } from 'react';
import { Play, Pause, Volume2, VolumeX } from 'lucide-react';

// Mock type if shared not available yet, or import
// import { ClipMetadata } from "@squadsync/shared";

export interface WebClip {
  id: string;
  url: string;
  author: string;
  offsetMs: number; // Offset from the start of the "session"
}

interface WebSquadGridProps {
  clips: WebClip[];
}

export function WebSquadGrid({ clips }: WebSquadGridProps) {
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [duration, setDuration] = useState(0);
  const [muted, setMuted] = useState(true);

  const videoRefs = useRef<Map<string, HTMLVideoElement>>(new Map());

  // Sync Logic
  const handlePlay = () => {
    setIsPlaying(true);
    videoRefs.current.forEach((video) => video.play());
  };

  const handlePause = () => {
    setIsPlaying(false);
    videoRefs.current.forEach((video) => video.pause());
  };

  const handleSeek = (time: number) => {
    setCurrentTime(time);
    videoRefs.current.forEach((video) => {
      video.currentTime = time;
    });
  };

  // Update time from "Leader" (first video)
  const handleTimeUpdate = (e: React.SyntheticEvent<HTMLVideoElement>) => {
    if (clips.length > 0 && e.currentTarget.src === clips[0].url) {
      setCurrentTime(e.currentTarget.currentTime);
      setDuration(e.currentTarget.duration || 0);
    }
  };

  return (
    <div className="flex flex-col h-full bg-slate-950">
      {/* Grid */}
      <div className="flex-1 grid grid-cols-2 gap-4 p-4 overflow-hidden">
        {clips.map((clip, index) => (
          <div
            key={clip.id}
            className="relative group rounded-xl overflow-hidden bg-slate-900 border border-white/10"
          >
            <video
              ref={(el) => {
                if (el) videoRefs.current.set(clip.id, el);
                else videoRefs.current.delete(clip.id);
              }}
              src={clip.url}
              className="w-full h-full object-contain"
              muted={muted} // Start muted for autoplay policy
              onTimeUpdate={index === 0 ? handleTimeUpdate : undefined}
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
            max={duration || 100}
            value={currentTime}
            onChange={(e) => handleSeek(parseFloat(e.target.value))}
            className="w-full h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer accent-indigo-500"
          />
          <div className="flex justify-between text-xs text-slate-400 font-mono">
            <span>{formatTime(currentTime)}</span>
            <span>{formatTime(duration)}</span>
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
