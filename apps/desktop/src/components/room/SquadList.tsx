import React from 'react';
import { RoomState, RoomMember } from '@squadsync/shared';
import { Users, LogOut } from 'lucide-react';

interface SquadListProps {
  roomState: RoomState;
  currentUserId: string;
  onLeave: () => void;
}

export const SquadList: React.FC<SquadListProps> = ({ roomState, currentUserId, onLeave }) => {
  return (
    <div className="w-full max-w-sm mx-auto p-6 bg-slate-900/50 backdrop-blur-xl rounded-2xl border border-white/10 shadow-xl">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-3 text-emerald-400">
          <Users size={24} />
          <div>
            <h2 className="text-xl font-bold text-white">Squad Active</h2>
            <p className="text-xs text-slate-400 font-mono">ID: {roomState.id}</p>
          </div>
        </div>
        <button
          onClick={onLeave}
          className="p-2 text-slate-400 hover:text-red-400 hover:bg-red-500/10 rounded-lg transition-colors"
          title="Leave Squad"
        >
          <LogOut size={20} />
        </button>
      </div>

      <div className="space-y-3">
        {roomState.members.map((member: RoomMember) => (
          <div
            key={member.userId}
            className={`flex items-center justify-between p-3 rounded-xl border ${
              member.userId === currentUserId
                ? 'bg-indigo-500/10 border-indigo-500/30'
                : 'bg-slate-950/30 border-slate-800'
            }`}
          >
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 rounded-full bg-slate-800 flex items-center justify-center text-sm font-bold text-slate-300">
                {member.displayName.charAt(0).toUpperCase()}
              </div>
              <div>
                <p className="text-sm font-medium text-white">
                  {member.displayName}
                  {member.userId === currentUserId && (
                    <span className="ml-2 text-xs text-indigo-400 font-normal">(You)</span>
                  )}
                </p>
                <p className="text-xs text-slate-500">
                  {member.isRecording ? 'Recording' : 'Idle'}
                </p>
              </div>
            </div>
            <div
              className={`w-2 h-2 rounded-full ${
                member.isRecording ? 'bg-red-500 animate-pulse' : 'bg-emerald-500'
              }`}
            />
          </div>
        ))}
      </div>
    </div>
  );
};
