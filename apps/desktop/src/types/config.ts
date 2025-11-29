export interface RecordingConfig {
  path: string;
  resolution: string | null;
  framerate: number;
  bitrate: string;
  encoder: string;
  mode: string;
  capture_method: string;
  audio_source: string | null;
  audio_codec: string | null;
  container_format: string | null;
  segment_time: number;
  segment_wrap: number;
}

export interface AppConfig {
  recording: {
    path: string;
    resolution?: string;
    framerate: number;
    bitrate: string;
    encoder: string;
    audio_source?: string;
    audio_codec?: string;
  };
}
