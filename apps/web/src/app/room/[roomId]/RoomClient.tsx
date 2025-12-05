'use client';

import { useEffect, useState } from 'react';
import { WebSquadGrid } from '@/components/WebSquadGrid';
import { Loader2 } from 'lucide-react';
import PartySocket from 'partysocket';

import { View } from '@squadsync/shared';
import { logger } from '@/lib/logger';

interface RoomClientProps {
  roomId: string;
}

interface RoomClip {
  id: string;
  timestamp?: number;
  views?: View[];
  [key: string]: unknown;
}

const PARTYKIT_HOST = process.env.NEXT_PUBLIC_PARTYKIT_HOST || 'localhost:1999';

export default function RoomClient({ roomId }: RoomClientProps) {
  const [rawClips, setRawClips] = useState<RoomClip[]>([]);
  const [loading, setLoading] = useState(true);

  // Derive flat list of playable clips (views) for the grid
  const clips: View[] = rawClips.flatMap((clip) => clip.views || []);

  useEffect(() => {
    const socket = new PartySocket({
      host: PARTYKIT_HOST,
      room: roomId,
    });

    socket.addEventListener('message', (event) => {
      const msg = JSON.parse(event.data);
      logger.info('Web Client received:', msg);

      if (msg.type === 'CLIP_LIST') {
        setRawClips(msg.clips);
        setLoading(false);
      } else if (msg.type === 'START_CLIP') {
        // Add placeholder for new clip (initially no views)
        setRawClips((prev) => [
          ...prev,
          {
            id: msg.clipId,
            timestamp: msg.referenceTime,
            views: [],
          },
        ]);
      } else if (msg.type === 'CLIP_UPDATED') {
        setRawClips((prev) => {
          const index = prev.findIndex((c) => c.id === msg.clipId);
          if (index === -1) return prev;

          const newClips = [...prev];
          const clip = { ...newClips[index] };

          // Update or add the view
          const views = [...(clip.views || [])];
          const viewIndex = views.findIndex((v: View) => v.author === msg.view.author);

          if (viewIndex !== -1) {
            views[viewIndex] = msg.view;
          } else {
            views.push(msg.view);
          }

          clip.views = views;
          newClips[index] = clip;
          return newClips;
        });
      }
    });

    return () => socket.close();
  }, [roomId]);

  if (loading && clips.length === 0) {
    return (
      <div className="h-screen w-full flex items-center justify-center bg-slate-950 text-white">
        <div className="flex flex-col items-center gap-4">
          <Loader2 className="animate-spin text-indigo-500" size={48} />
          <p className="text-slate-400 font-medium">Connecting to Room...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-screen w-full bg-slate-950">
      <WebSquadGrid clips={clips} />
    </div>
  );
}
