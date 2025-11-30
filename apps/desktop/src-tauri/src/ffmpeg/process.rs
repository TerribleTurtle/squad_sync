use tauri::{AppHandle, Manager};
use crate::ffmpeg::commands::FfmpegCommandBuilder;
use crate::ffmpeg::encoder::{self, VideoEncoder};
use crate::ffmpeg::session::RecordingSession;
use crate::state::RecordingState;
use crate::state::RecordingMessage;
use std::sync::mpsc::Sender;

pub async fn start_recording_process(app: &AppHandle) -> Result<Sender<RecordingMessage>, String> {
    let _app_cache = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    let state = app.state::<RecordingState>();
    let config = state.config.lock().map_err(|e| e.to_string())?.clone(); 

    // 1. Determine Output Path (Temp Buffer)
    // Expand %TEMP% if present
    let temp_path_str = config.recording.temp_path.replace("%TEMP%", &std::env::temp_dir().to_string_lossy());
    let buffer_dir = std::path::PathBuf::from(temp_path_str);
    
    if !buffer_dir.exists() {
        std::fs::create_dir_all(&buffer_dir).map_err(|e| e.to_string())?;
    }

    // Segment Pattern: clip_%03d.ts
    let output_pattern = buffer_dir.join("clip_%03d.ts").to_string_lossy().to_string();
    let playlist_path = buffer_dir.join("buffer.m3u8").to_string_lossy().to_string();

    // Calculate Wrap Limit
    let segment_time = config.recording.segment_time;
    let buffer_duration = config.recording.buffer_duration;
    let wrap_limit = (buffer_duration / segment_time) + 1; // +1 for safety overlap

    println!("Buffer Dir: {:?}", buffer_dir);
    println!("Segment Pattern: {}", output_pattern);
    println!("Playlist: {}", playlist_path);
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
        (1920, 1080, 0, 0) // Fallback
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
                // Smart Bypass: If target matches monitor, disable scaler
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
        // 1080p60 -> ~12Mbps
        // 1440p60 -> ~22Mbps
        let pixels = target_width as u64 * target_height as u64;
        let fps = config.recording.framerate as u64;
        let bits_per_sec = (pixels * fps) / 10;
        format!("{}k", bits_per_sec / 1000)
    };

    println!("Configuring Recording: {}x{} @ {}fps, Bitrate: {}, Scaler: {}", 
        target_width, target_height, config.recording.framerate, bitrate, use_scaler);

    // 5. System Audio Setup (Just get the device name)
    let system_audio_device = config.recording.system_audio_device.clone();
    let system_audio_enabled = system_audio_device.is_some();
    // We don't start capture here anymore, just pass the device name
    // Default sample rate 48000, will be detected in session
    let system_sample_rate = 48000; 
    
    // 6. Build Command
    let builder = FfmpegCommandBuilder::new(output_pattern)
        .with_video_codec(encoder.as_ffmpeg_codec().to_string())
        .with_preset(config.recording.video_preset.clone())
        .with_tune(config.recording.video_tune.clone())
        .with_profile(config.recording.video_profile.clone())
        .with_bitrate(bitrate)
        .with_framerate(config.recording.framerate)
        .with_resolution(if use_scaler { Some(format!("{}x{}", target_width, target_height)) } else { None })
        .with_video_size(format!("{}x{}", width, height))
        .with_monitor_index(config.recording.monitor_index)
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
    )
}
