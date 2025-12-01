
// Audio Pipes
pub const SYSTEM_AUDIO_PIPE_NAME: &str = r"\\.\pipe\squad_sync_system_audio";
pub const MIC_AUDIO_PIPE_NAME: &str = r"\\.\pipe\squad_sync_mic_audio";

// Audio Defaults
pub const DEFAULT_AUDIO_SAMPLE_RATE: u32 = 48000;
pub const DEFAULT_AUDIO_CHANNELS: u16 = 2;
pub const DEFAULT_MIC_CHANNELS: u16 = 1;
pub const DEFAULT_AUDIO_BITRATE: &str = "192k";
pub const DEFAULT_AUDIO_CODEC: &str = "aac";
pub const AUDIO_LATENCY_THRESHOLD_MS: u64 = 20;
pub const AUDIO_SILENCE_TIMEOUT_MS: u64 = 20;

// Video Defaults
pub const DEFAULT_VIDEO_CODEC: &str = "libx264";
pub const DEFAULT_VIDEO_BITRATE: &str = "15M";
pub const DEFAULT_VIDEO_FRAMERATE: u32 = 60;
pub const DEFAULT_VIDEO_PRESET: &str = "faster";
pub const DEFAULT_VIDEO_TUNE: &str = "zerolatency"; // For CPU encoding
pub const DEFAULT_VIDEO_PROFILE: &str = "high";

// FFmpeg Configuration
pub const FFMPEG_THREAD_QUEUE_SIZE: &str = "2048"; // For video
pub const FFMPEG_AUDIO_THREAD_QUEUE_SIZE: &str = "4096"; // For audio
pub const FFMPEG_EXTRA_HW_FRAMES: &str = "8";
pub const FFMPEG_MAX_MUXING_QUEUE_SIZE: &str = "9999";

// Buffer / Latency
pub const AUDIO_BUFFER_SIZE_MS: &str = "50";
pub const RTBUFSIZE: &str = "100M";

// Presets & Tunes
pub const PRESET_P1: &str = "p1";
pub const PRESET_P4: &str = "p4";
pub const PRESET_SPEED: &str = "speed";
pub const PRESET_BALANCED: &str = "balanced";
pub const PRESET_VERYFAST: &str = "veryfast";
pub const PRESET_ULTRAFAST: &str = "ultrafast";
pub const TUNE_ULL: &str = "ull";
pub const TUNE_ZEROLATENCY: &str = "zerolatency";
pub const PROFILE_HIGH: &str = "high";

// System / Errors
pub const ERROR_NO_DATA: i32 = 232; // Windows Error 232: The pipe is being closed.

// Segment / Output
pub const SEGMENT_LIST_SIZE: &str = "0";
pub const SEGMENT_LIST_TYPE: &str = "m3u8";
pub const SEGMENT_FORMAT_MKV: &str = "matroska";
pub const OUTPUT_FORMAT_SEGMENT: &str = "segment";
pub const OUTPUT_FORMAT_MP4: &str = "mp4";
pub const OUTPUT_FORMAT_LAVFI: &str = "lavfi";
pub const OUTPUT_FORMAT_DSHOW: &str = "dshow";
pub const OUTPUT_FORMAT_F32LE: &str = "f32le";

// FFmpeg Analysis
pub const FFMPEG_ANALYZE_DURATION: &str = "2147483647";
pub const FFMPEG_PROBESIZE: &str = "2147483647";
// Bitrate / GOP
pub const BITRATE_MAX_MULTIPLIER: u32 = 3;
pub const BITRATE_MAX_DIVISOR: u32 = 2;
pub const BITRATE_BUF_MULTIPLIER: u32 = 2;
pub const GOP_MULTIPLIER: u32 = 1;

// Defaults
pub const DEFAULT_WIDTH: u32 = 1920;
pub const DEFAULT_HEIGHT: u32 = 1080;

// Replay Logic
pub const REPLAY_WAIT_RETRIES: u32 = 15;
pub const REPLAY_WAIT_DELAY_MS: u64 = 1000;
pub const REPLAY_AUDIO_SYNC_RETRIES: u32 = 5;
pub const REPLAY_AUDIO_SYNC_THRESHOLD_SEC: f64 = 0.5;
pub const REPLAY_SEGMENT_AGE_THRESHOLD_SEC: u64 = 5;
pub const REPLAY_COPY_RETRIES: u32 = 20;
pub const REPLAY_COPY_DELAY_MS: u64 = 50;
pub const REPLAY_FLUSH_WAIT_MS: u64 = 500;
