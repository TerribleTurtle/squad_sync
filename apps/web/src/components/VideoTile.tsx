'use client';

import { memo, useEffect, useRef } from 'react';
import { View } from '@squadsync/shared';

interface VideoTileProps {
  clip: View;
  muted: boolean;
  onMount: (url: string, el: HTMLVideoElement) => void;
  onUnmount: (url: string) => void;
  onWaiting: (url: string) => void;
  onPlaying: (url: string) => void;
  // Included to trigger re-renders if these change, though usually they don't for existing clips
}

export const VideoTile = memo(function VideoTile({
  clip,
  muted,
  onMount,
  onUnmount,
  onWaiting,
  onPlaying,
}: VideoTileProps) {
  const videoRef = useRef<HTMLVideoElement | null>(null);

  useEffect(() => {
    const el = videoRef.current;
    if (!el) return;

    onMount(clip.url, el);

    const handleWaiting = () => onWaiting(clip.url);
    const handlePlaying = () => onPlaying(clip.url);
    const handleCanPlay = () => onPlaying(clip.url); // Treat canplay as not waiting

    el.addEventListener('waiting', handleWaiting);
    el.addEventListener('playing', handlePlaying);
    el.addEventListener('canplay', handleCanPlay);

    return () => {
      el.removeEventListener('waiting', handleWaiting);
      el.removeEventListener('playing', handlePlaying);
      el.removeEventListener('canplay', handleCanPlay);
      onUnmount(clip.url);
    };
  }, [clip.url, onMount, onUnmount, onWaiting, onPlaying]);

  return (
    <div className="relative group rounded-xl overflow-hidden bg-slate-900 border border-white/10 w-full h-full">
      <video
        ref={videoRef}
        src={clip.url}
        className="w-full h-full object-contain"
        muted={muted}
        playsInline
        preload="auto"
      />
      <div className="absolute bottom-4 left-4 px-3 py-1 rounded-full bg-black/60 backdrop-blur-md text-white text-xs font-medium">
        {clip.author}
      </div>
    </div>
  );
});
