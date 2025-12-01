//! FFmpeg Process Manager
//! 
//! This module orchestrates the recording process. It handles:
//! 1. Configuration resolution (resolution, bitrate, encoder).
//! 2. Temp buffer management.
//! 3. Command construction via [crate::ffmpeg::commands::FfmpegCommandBuilder].
//! 4. Session spawning via [crate::ffmpeg::session::RecordingSession].

use tauri::{AppHandle, Manager};
use crate::ffmpeg::commands::FfmpegCommandBuilder;
use crate::ffmpeg::encoder::{self, VideoEncoder};
use crate::ffmpeg::session::RecordingSession;
use crate::state::RecordingState;
use crate::state::RecordingMessage;
use std::sync::mpsc::Sender;
use crate::constants::{
    DEFAULT_AUDIO_SAMPLE_RATE, DEFAULT_AUDIO_CHANNELS,
    DEFAULT_WIDTH, DEFAULT_HEIGHT
};

pub async fn start_recording_process(app: &AppHandle) -> Result<Sender<RecordingMessage>, String> {
    let _app_cache = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    let state = app.state::<RecordingState>();
    let config = state.config.lock().map_err(|e| e.to_string())?.clone(); 

    // 1. Determine Output Path (Temp Buffer)
    let temp_path_str = config.recording.temp_path.replace("%TEMP%", &std::env::temp_dir().to_string_lossy());
    let buffer_dir = std::path::PathBuf::from(temp_path_str);
    
    if buffer_dir.exists() {
        let _ = std::fs::remove_dir_all(&buffer_dir);
    }
    std::fs::create_dir_all(&buffer_dir).map_err(|e| e.to_string())?;

    let output_pattern = buffer_dir.join("clip_%03d.mkv").to_string_lossy().to_string();
    let playlist_path = buffer_dir.join("buffer.m3u8").to_string_lossy().to_string();

    let segment_time = config.recording.segment_time;
    let buffer_duration = config.recording.buffer_duration;
    let wrap_limit = (buffer_duration / segment_time) + 1;

    println!("Buffer Dir: {:?}", buffer_dir);
    let video_pattern = buffer_dir.join("video_%03d.mkv").to_string_lossy().to_string();
    let audio_pattern = buffer_dir.join("audio_%03d.mkv").to_string_lossy().to_string();

    println!("Buffer Dir: {:?}", buffer_dir);
    println!("Video Pattern: {}", video_pattern);
    println!("Audio Pattern: {}", audio_pattern);
    println!("Wrap Limit: {}", wrap_limit);

    // 2. Select Encoder
    let encoder = if config.recording.encoder == "auto" {
        encoder::get_best_encoder()
    } else {
        match config.recording.encoder.as_str() {
            "h264_nvenc" => VideoEncoder::Nvenc,
            "h264_amf" => VideoEncoder::Amf,
            "h264_qsv" => VideoEncoder::Qsv,
            "h264_vaapi" => VideoEncoder::Vaapi,
            _ => VideoEncoder::X264,
        }
    };
    println!("Selected encoder: {:?}", encoder);

    // 3. Get Monitor Info
    let monitor = if let Some(window) = app.get_webview_window("main") {
        let monitors = window.available_monitors().unwrap_or_default();
        let index = config.recording.monitor_index as usize;
        if index < monitors.len() {
            Some(monitors[index].clone())
        } else {
            window.primary_monitor().ok().flatten()
        }
    } else {
        None
    };

    let (width, height, _x, _y) = if let Some(m) = monitor {
        let size = m.size();
        let pos = m.position();
        (size.width, size.height, pos.x, pos.y)
    } else {
        (DEFAULT_WIDTH, DEFAULT_HEIGHT, 0, 0)
    };

    // 4. Smart Resolution & Bitrate Logic
    let (target_width, target_height, use_scaler) = if let Some(res_str) = &config.recording.resolution {
        if res_str.to_lowercase() == "native" {
            (width, height, false)
        } else {
            let parts: Vec<&str> = res_str.split('x').collect();
            if parts.len() == 2 {
                let w = parts[0].parse::<u32>().unwrap_or(width);
                let h = parts[1].parse::<u32>().unwrap_or(height);
                if w == width && h == height {
                    (width, height, false)
                } else {
                    (w, h, true)
                }
            } else {
                (width, height, false)
            }
        }
    } else {
        (width, height, false)
    };

    let bitrate = if let Some(b) = &config.recording.bitrate {
        b.clone()
    } else {
        // Dynamic Bitrate: (Pixels * FPS) / 10 -> 0.1 bits per pixel
        // See [crate::ffmpeg::utils::calculate_dynamic_bitrate]
        crate::ffmpeg::utils::calculate_dynamic_bitrate(target_width, target_height, config.recording.framerate)
    };

    println!("Configuring Recording: {}x{} @ {}fps, Bitrate: {}, Scaler: {}", 
        target_width, target_height, config.recording.framerate, bitrate, use_scaler);

    let system_audio_device = config.recording.system_audio_device.clone();
    let system_audio_enabled = system_audio_device.is_some();
    let system_sample_rate = DEFAULT_AUDIO_SAMPLE_RATE; 
    
    // 6. Build Command
    let builder = FfmpegCommandBuilder::new(output_pattern)
        .with_video_codec(encoder.as_ffmpeg_codec().to_string())
        .with_preset(config.recording.video_preset.clone())
        .with_tune(config.recording.video_tune.clone())
        .with_profile(config.recording.video_profile.clone())
        .with_bitrate(bitrate.clone())
        .with_framerate(config.recording.framerate)
        .with_resolution(if use_scaler { Some(format!("{}x{}", target_width, target_height)) } else { None })
        .with_video_size(format!("{}x{}", width, height))
        .with_monitor_index(config.recording.monitor_index)
        .with_audio_source(config.recording.audio_source.clone())
        .with_system_audio(system_audio_enabled)
        .with_audio_input_config(system_sample_rate, None, None, None)
        .with_audio_output_config(Some("pcm_s16le".to_string()), config.recording.audio_bitrate.clone(), DEFAULT_AUDIO_SAMPLE_RATE, DEFAULT_AUDIO_CHANNELS)
        .with_audio_backend(config.recording.audio_backend.clone())
        .with_segment_config(segment_time, wrap_limit, playlist_path);

    // 7. Spawn Session
    RecordingSession::spawn(
        app.clone(),
        builder,
        config.recording.audio_source,
        system_audio_enabled,
        system_sample_rate,
        system_audio_device,
        config.recording.audio_codec,
        config.recording.audio_bitrate,
        bitrate,
        buffer_dir,
        config.recording.buffer_retention_seconds,
        config.recording.audio_backend,
    )
}
