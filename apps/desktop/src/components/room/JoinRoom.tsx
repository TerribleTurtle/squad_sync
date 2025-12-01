import React, { useState } from 'react';
import { Users, ArrowRight, Plus } from 'lucide-react';

interface JoinRoomProps {
  onJoin: (roomId: string, displayName: string) => void;
}

export const JoinRoom: React.FC<JoinRoomProps> = ({ onJoin }) => {
  const [roomId, setRoomId] = useState('');
  const [displayName, setDisplayName] = useState('');

  const handleJoin = (e: React.FormEvent) => {
    e.preventDefault();
    if (roomId && displayName) {
      onJoin(roomId, displayName);
    }
  };

  const handleCreate = () => {
    const randomId = Math.random().toString(36).substring(2, 8).toUpperCase();
    if (displayName) {
      onJoin(randomId, displayName);
    }
  };

  return (
    <div className="w-full max-w-sm mx-auto p-6 bg-slate-900/50 backdrop-blur-xl rounded-2xl border border-white/10 shadow-xl">
      <div className="flex items-center gap-3 mb-6 text-indigo-400">
        <Users size={24} />
        <h2 className="text-xl font-bold text-white">Join Squad</h2>
      </div>

      <form onSubmit={handleJoin} className="flex flex-col gap-4">
        <div>
          <label className="block text-xs font-medium text-slate-400 mb-1.5 uppercase tracking-wider">
            Display Name
          </label>
          <input
            type="text"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            placeholder="Enter your name"
            className="w-full px-4 py-3 bg-slate-950/50 border border-slate-700/50 rounded-xl text-white placeholder-slate-600 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 transition-all"
            required
          />
        </div>

        <div>
          <label className="block text-xs font-medium text-slate-400 mb-1.5 uppercase tracking-wider">
            Room ID
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={roomId}
              onChange={(e) => setRoomId(e.target.value.toUpperCase())}
              placeholder="e.g. X7K9P2"
              className="w-full px-4 py-3 bg-slate-950/50 border border-slate-700/50 rounded-xl text-white placeholder-slate-600 focus:outline-none focus:ring-2 focus:ring-indigo-500/50 focus:border-indigo-500/50 transition-all font-mono uppercase"
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3 mt-2">
          <button
            type="button"
            onClick={handleCreate}
            disabled={!displayName}
            className="flex items-center justify-center gap-2 px-4 py-3 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-xl font-medium transition-all disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Plus size={18} />
            <span>Create</span>
          </button>
          <button
            type="submit"
            disabled={!roomId || !displayName}
            className="flex items-center justify-center gap-2 px-4 py-3 bg-indigo-600 hover:bg-indigo-500 text-white rounded-xl font-medium shadow-lg shadow-indigo-500/20 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <span>Join</span>
            <ArrowRight size={18} />
          </button>
        </div>
      </form>
    </div>
  );
};
