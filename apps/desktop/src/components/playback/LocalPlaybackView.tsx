import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Search, RefreshCw, FolderOpen } from 'lucide-react';
import { ClipCard } from './ClipCard';
import { logger } from '../../lib/logger';

interface Recording {
  name: string;
  path: string;
  thumbnail_path?: string;
  size: number;
  created_at: number;
  duration?: number;
}

export function LocalPlaybackView() {
  const [recordings, setRecordings] = useState<Recording[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState('');

  const fetchRecordings = async (showLoading = true) => {
    if (showLoading) setIsLoading(true);
    try {
      const data = await invoke<Recording[]>('get_recordings');
      setRecordings(data);

      // Generate thumbnails for missing ones
      data.forEach(async (rec) => {
        if (!rec.thumbnail_path) {
          try {
            const thumbPath = await invoke<string>('generate_thumbnail', { path: rec.path });
            setRecordings((prev) =>
              prev.map((p) => (p.path === rec.path ? { ...p, thumbnail_path: thumbPath } : p))
            );
          } catch (e) {
            logger.error('Failed to generate thumbnail for', rec.name, e);
          }
        }
      });
    } catch (error) {
      logger.error('Failed to fetch recordings:', error);
    } finally {
      if (showLoading) setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchRecordings();
  }, []);

  const filteredRecordings = recordings.filter((rec) =>
    rec.name.toLowerCase().includes(searchQuery.toLowerCase())
  );

  const openRecordingsFolder = async () => {
    // We can get the config to find the path, or just ask backend to open the dir of the first file,
    // or better yet, add a specific command to open the output dir.
    // For now, let's try to open the folder of the first recording if it exists,
    // or fallback to a generic open command if we had one (we don't yet).
    // Actually, let's just use the first recording's path to open the folder.
    if (recordings.length > 0) {
      await invoke('show_in_folder', { path: recordings[0].path });
    } else {
      // Fallback: Fetch config to get path (not implemented in this view yet)
      // For now do nothing or show toast
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between mb-6 pr-12">
        <div>
          <h2 className="text-2xl font-bold text-white">Local Recordings</h2>
          <p className="text-slate-400 text-sm">Manage and watch your captured clips</p>
        </div>

        <div className="flex items-center gap-3">
          <div className="relative">
            <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-slate-500" />
            <input
              type="text"
              placeholder="Search clips..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="bg-slate-800/50 border border-white/5 rounded-xl pl-9 pr-4 py-2 text-sm text-white focus:outline-none focus:ring-1 focus:ring-indigo-500 w-64 transition-all"
            />
          </div>

          <button
            onClick={() => fetchRecordings(true)}
            className="p-2 text-slate-400 hover:text-white hover:bg-white/5 rounded-xl transition-colors"
            title="Refresh"
          >
            <RefreshCw size={20} className={isLoading ? 'animate-spin' : ''} />
          </button>

          <button
            onClick={openRecordingsFolder}
            className="p-2 text-slate-400 hover:text-indigo-400 hover:bg-indigo-500/10 rounded-xl transition-colors"
            title="Open Recordings Folder"
          >
            <FolderOpen size={20} />
          </button>
        </div>
      </div>

      {/* Grid */}
      <div className="flex-1 overflow-y-auto pr-2 -mr-2 scrollbar-thin scrollbar-thumb-slate-700 scrollbar-track-transparent">
        {isLoading ? (
          <div className="flex items-center justify-center h-64">
            <div className="w-8 h-8 border-2 border-indigo-500 border-t-transparent rounded-full animate-spin"></div>
          </div>
        ) : filteredRecordings.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-64 text-slate-500">
            <div className="w-16 h-16 rounded-full bg-slate-800/50 flex items-center justify-center mb-4">
              <FolderOpen size={32} className="opacity-50" />
            </div>
            <p className="text-lg font-medium">No recordings found</p>
            <p className="text-sm opacity-70">
              {searchQuery ? 'Try a different search term' : 'Start recording to see clips here'}
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 pb-4">
            {filteredRecordings.map((rec) => (
              <ClipCard
                key={rec.path}
                recording={rec}
                onDelete={() => fetchRecordings(false)}
                onRename={() => fetchRecordings(false)}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
