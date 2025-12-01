export interface PluginEvent {
  eventId: string;
  timestamp: number;
  game: string;
  type: string;
  metadata: Record<string, unknown>;
}

export interface ClipTrigger {
  pluginId: string;
  timestamp: number;
  duration: number;
  label: string;
}
