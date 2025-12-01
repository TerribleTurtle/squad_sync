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
  user: {
    display_name: string | null;
    user_id: string | null;
  };
  recording: {
    path: string;
    resolution?: string;
    framerate: number;
    bitrate: string;
    monitor_index: number;
    video_profile?: string;
    audio_bitrate?: string;
    mic_audio_delay?: number;
    encoder: string;
    audio_source?: string;
    system_audio_device?: string;
    audio_codec?: string;
    buffer_duration?: number;
    segment_time?: number;
  };
}
