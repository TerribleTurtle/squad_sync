'use client';

import { useEffect, useState } from 'react';
import { WebSquadGrid, WebClip } from '@/components/WebSquadGrid';
import { Loader2 } from 'lucide-react';
import PartySocket from 'partysocket';

interface RoomClientProps {
  roomId: string;
}

const PARTYKIT_HOST = process.env.NEXT_PUBLIC_PARTYKIT_HOST || 'localhost:1999';

export default function RoomClient({ roomId }: RoomClientProps) {
  const [clips, setClips] = useState<WebClip[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const socket = new PartySocket({
      host: PARTYKIT_HOST,
      room: roomId,
    });

    socket.addEventListener('message', (event) => {
      const msg = JSON.parse(event.data);
      console.log('Web Client received:', msg);

      if (msg.type === 'CLIP_LIST') {
        setClips(msg.clips);
        setLoading(false);
      } else if (msg.type === 'START_CLIP') {
        if (msg.playbackUrl) {
          setClips((prev) => [
            ...prev,
            {
              id: msg.clipId,
              url: msg.playbackUrl,
              author: 'New Clip',
              offsetMs: 0,
            },
          ]);
        }
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
