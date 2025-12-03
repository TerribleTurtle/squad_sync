import { useState, useEffect } from 'react';
import { Play, Trash2, FolderOpen, Edit2, Check, X } from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { readFile } from '@tauri-apps/plugin-fs';
import { ask } from '@tauri-apps/plugin-dialog';
import { formatDistanceToNow } from 'date-fns';

interface Recording {
  name: string;
  path: string;
  thumbnail_path?: string;
  size: number;
  created_at: number;
  duration?: number;
}

interface ClipCardProps {
  recording: Recording;
  onDelete: () => void;
  onRename: () => void;
}

export function ClipCard({ recording, onDelete, onRename }: ClipCardProps) {
  const [isRenaming, setIsRenaming] = useState(false);
  const [newName, setNewName] = useState(recording.name);
  const [isHovered, setIsHovered] = useState(false);
  const [thumbnailUrl, setThumbnailUrl] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    const loadThumbnail = async () => {
      if (recording.thumbnail_path) {
        try {
          const contents = await readFile(recording.thumbnail_path);
          if (active) {
            const blob = new Blob([contents], { type: 'image/jpeg' });
            const url = URL.createObjectURL(blob);
            setThumbnailUrl(url);
          }
        } catch (error) {
          console.error('Failed to load thumbnail:', error);
        }
      }
    };
    loadThumbnail();
    return () => {
      active = false;
      if (thumbnailUrl) URL.revokeObjectURL(thumbnailUrl);
    };
  }, [recording.thumbnail_path]);

  const formatSize = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const handlePlay = async () => {
    try {
      await invoke('open_file', { path: recording.path });
    } catch (error) {
      console.error('Failed to open file:', error);
    }
  };

  const handleShowInFolder = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await invoke('show_in_folder', { path: recording.path });
    } catch (error) {
      console.error('Failed to show in folder:', error);
    }
  };

  const handleDelete = async (e: React.MouseEvent) => {
    e.stopPropagation();
    const confirmed = await ask(`Are you sure you want to delete ${recording.name}?`, {
      title: 'Delete Recording',
      kind: 'warning',
    });

    if (confirmed) {
      try {
        await invoke('delete_recording', { path: recording.path });
        onDelete();
      } catch (error) {
        console.error('Failed to delete recording:', error);
      }
    }
  };

  const handleRename = async (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!newName.trim() || newName === recording.name) {
      setIsRenaming(false);
      setNewName(recording.name);
      return;
    }

    try {
      await invoke('rename_recording', { path: recording.path, newName: newName.trim() });
      setIsRenaming(false);
      onRename();
    } catch (error) {
      console.error('Failed to rename recording:', error);
      alert('Failed to rename recording: ' + error);
    }
  };

  return (
    <div
      className="group relative bg-slate-800/50 border border-white/5 rounded-xl overflow-hidden hover:border-indigo-500/30 transition-all duration-300 hover:shadow-lg hover:shadow-indigo-500/10"
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {/* Thumbnail */}
      <div
        className="aspect-video bg-slate-900 relative cursor-pointer group-hover:opacity-90 transition-opacity"
        onClick={handlePlay}
      >
        {thumbnailUrl ? (
          <img src={thumbnailUrl} alt={recording.name} className="w-full h-full object-cover" />
        ) : (
          <div className="absolute inset-0 flex items-center justify-center bg-slate-800">
            <span className="text-slate-600 text-xs">No Preview</span>
          </div>
        )}

        <div className="absolute inset-0 flex items-center justify-center">
          <div className="w-12 h-12 rounded-full bg-white/10 backdrop-blur-sm flex items-center justify-center text-white opacity-0 group-hover:opacity-100 transition-all duration-300 transform group-hover:scale-110">
            <Play size={20} className="fill-current ml-1" />
          </div>
        </div>

        {/* Duration Badge (Placeholder) */}
        <div className="absolute bottom-2 right-2 px-1.5 py-0.5 rounded bg-black/60 text-[10px] font-medium text-white backdrop-blur-sm">
          VIDEO
        </div>
      </div>

      {/* Content */}
      <div className="p-3">
        <div className="flex items-start justify-between gap-2 mb-1">
          {isRenaming ? (
            <div className="flex items-center gap-1 w-full" onClick={(e) => e.stopPropagation()}>
              <input
                type="text"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                className="flex-1 bg-slate-900 border border-indigo-500/50 rounded px-2 py-1 text-xs text-white focus:outline-none focus:ring-1 focus:ring-indigo-500"
                autoFocus
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleRename(e as any);
                  if (e.key === 'Escape') {
                    setIsRenaming(false);
                    setNewName(recording.name);
                  }
                }}
              />
              <button
                onClick={handleRename}
                className="p-1 hover:bg-emerald-500/20 text-emerald-400 rounded transition-colors"
              >
                <Check size={14} />
              </button>
              <button
                onClick={() => {
                  setIsRenaming(false);
                  setNewName(recording.name);
                }}
                className="p-1 hover:bg-red-500/20 text-red-400 rounded transition-colors"
              >
                <X size={14} />
              </button>
            </div>
          ) : (
            <h3
              className="text-sm font-medium text-slate-200 truncate cursor-pointer hover:text-indigo-400 transition-colors"
              onClick={handlePlay}
              title={recording.name}
            >
              {recording.name}
            </h3>
          )}
        </div>

        <div className="flex items-center justify-between text-[10px] text-slate-500">
          <span>{formatSize(recording.size)}</span>
          <span>{formatDistanceToNow(recording.created_at * 1000, { addSuffix: true })}</span>
        </div>

        {/* Actions */}
        <div
          className={`flex items-center justify-end gap-1 mt-3 pt-2 border-t border-white/5 transition-opacity duration-200 ${isHovered ? 'opacity-100' : 'opacity-0'}`}
        >
          <button
            onClick={handleShowInFolder}
            className="p-1.5 text-slate-400 hover:text-indigo-400 hover:bg-indigo-500/10 rounded-lg transition-colors"
            title="Show in Folder"
          >
            <FolderOpen size={14} />
          </button>
          <button
            onClick={(e) => {
              e.stopPropagation();
              setIsRenaming(true);
            }}
            className="p-1.5 text-slate-400 hover:text-amber-400 hover:bg-amber-500/10 rounded-lg transition-colors"
            title="Rename"
          >
            <Edit2 size={14} />
          </button>
          <button
            onClick={handleDelete}
            className="p-1.5 text-slate-400 hover:text-red-400 hover:bg-red-500/10 rounded-lg transition-colors"
            title="Delete"
          >
            <Trash2 size={14} />
          </button>
        </div>
      </div>
    </div>
  );
}
