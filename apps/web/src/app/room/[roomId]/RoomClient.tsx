'use client';

import { useEffect, useState } from 'react';
import { WebSquadGrid, WebClip } from '@/components/WebSquadGrid';
import { Loader2 } from 'lucide-react';

// import PartySocket from "partysocket";

interface RoomClientProps {
  roomId: string;
}

export default function RoomClient({ roomId }: RoomClientProps) {
  const [clips, setClips] = useState<WebClip[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // TODO: Connect to PartyKit or Fetch from API
    // const socket = new PartySocket({ host: "...", room: roomId });

    // Mock Data for MVP Demo
    setTimeout(() => {
      setClips([
        {
          id: '1',
          url: 'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4', // Placeholder
          author: 'PlayerOne',
          offsetMs: 0,
        },
        {
          id: '2',
          url: 'https://commondatastorage.googleapis.com/gtv-videos-bucket/sample/ElephantsDream.mp4', // Placeholder
          author: 'PlayerTwo',
          offsetMs: 0,
        },
      ]);
      setLoading(false);
    }, 1000);
  }, [roomId]);

  if (loading) {
    return (
      <div className="h-screen w-full flex items-center justify-center bg-slate-950 text-white">
        <div className="flex flex-col items-center gap-4">
          <Loader2 className="animate-spin text-indigo-500" size={48} />
          <p className="text-slate-400 font-medium">Loading Replay...</p>
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
