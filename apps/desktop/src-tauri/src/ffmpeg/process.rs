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

pub async fn start_recording_process(app: &AppHandle) -> Result<(Sender<RecordingMessage>, std::thread::JoinHandle<()>), String> {
    let state = app.state::<RecordingState>();
    let config = state.config.lock().map_err(|e| e.to_string())?.clone(); 

    // 1. Determine Output Path (Temp Buffer)
    let temp_path_str = config.recording.temp_path.replace("%TEMP%", &std::env::temp_dir().to_string_lossy());
    let buffer_dir = std::path::PathBuf::from(temp_path_str);
    
    if buffer_dir.exists() {
        let _ = std::fs::remove_dir_all(&buffer_dir);
    }
    std::fs::create_dir_all(&buffer_dir).map_err(|e| e.to_string())?;

    // Note: output_pattern is overridden by session.rs for separate video/audio files
    let output_pattern = buffer_dir.join("clip_%03d.mkv").to_string_lossy().to_string();
    let playlist_path = buffer_dir.join("buffer.m3u8").to_string_lossy().to_string();

    let segment_time = config.recording.segment_time;
    let buffer_duration = config.recording.buffer_duration;
    let wrap_limit = (buffer_duration / segment_time) + 1;

    println!("Buffer Dir: {:?}", buffer_dir);
    // Patterns must match what session.rs uses (strftime format)
    // Added %03u for millisecond precision
    let video_pattern = buffer_dir.join("video_%Y%m%d%H%M%S%03u.mkv").to_string_lossy().to_string();
    let audio_pattern = buffer_dir.join("audio_%Y%m%d%H%M%S%03u.mkv").to_string_lossy().to_string();

    println!("Buffer Dir: {:?}", buffer_dir);
    println!("Video Pattern: {}", video_pattern);
    println!("Audio Pattern: {}", audio_pattern);
    println!("Wrap Limit: {}", wrap_limit);

    // 2. Select Encoder
    let encoder = if config.recording.encoder == "auto" {
        encoder::get_best_encoder(app)
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
    let (monitor, actual_monitor_index) = if let Some(window) = app.get_webview_window("main") {
        let monitors = window.available_monitors().unwrap_or_default();
        let requested_index_opt = config.recording.monitor_index;

        if let Some(idx) = requested_index_opt {
            let idx_usize = idx as usize;
            if idx_usize < monitors.len() {
                println!("Requested monitor index {} is valid.", idx);
                (Some(monitors[idx_usize].clone()), Some(idx))
            } else {
                println!("Requested monitor index {} is invalid. Falling back to Primary.", idx);
                // Fallback logic below
                find_primary_monitor(&window, &monitors)
            }
        } else {
            println!("No monitor index configured (Auto). Defaulting to Primary.");
            find_primary_monitor(&window, &monitors)
        }
    } else {
        println!("Failed to get main window for monitor detection.");
        (None, None)
    };



    // Fix: ddagrab uses DXGI indices which usually match the Display Number (DISPLAY1 -> 0)
    // Tauri's list index is arbitrary. We must parse the name to get the correct index.
    let ddagrab_index = if let Some(m) = &monitor {
        if let Some(name) = m.name() {
            println!("Inspecting monitor name for index: {}", name);
            if let Some(idx) = name.find("DISPLAY") {
                let num_part = &name[idx + 7..];
                let num_str: String = num_part.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(num) = num_str.parse::<u32>() {
                    if num > 0 {
                        println!("Parsed ddagrab index {} from name {}", num - 1, name);
                        num - 1
                    } else {
                        println!("Parsed invalid display number 0. Fallback to list index.");
                        actual_monitor_index.unwrap_or(0)
                    }
                } else {
                    println!("Failed to parse number from {}. Fallback to list index.", name);
                    actual_monitor_index.unwrap_or(0)
                }
            } else {
                println!("'DISPLAY' not found in name. Fallback to list index.");
                actual_monitor_index.unwrap_or(0)
            }
        } else {
            actual_monitor_index.unwrap_or(0)
        }
    } else {
        0
    };

    let (width, height, _x, _y) = if let Some(m) = &monitor {
        let size = m.size();
        let pos = m.position();
        (size.width, size.height, pos.x, pos.y)
    } else {
        (DEFAULT_WIDTH, DEFAULT_HEIGHT, 0, 0)
    };

    // 4. Smart Resolution & Bitrate Logic
    let scaling_mode = encoder::get_best_scaling_mode(app);

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
    println!("Scaling Mode: {:?}", scaling_mode);

    let system_audio_device = config.recording.system_audio_device.clone();
    let system_audio_enabled = system_audio_device.is_some();
    let system_sample_rate = DEFAULT_AUDIO_SAMPLE_RATE; 
    
    // 6. Build Command
    let builder = FfmpegCommandBuilder::new(output_pattern)
        .with_scaling_mode(scaling_mode)
        .with_video_codec(encoder.as_ffmpeg_codec().to_string())
        .with_preset(config.recording.video_preset.clone())
        .with_tune(config.recording.video_tune.clone())
        .with_profile(config.recording.video_profile.clone())
        .with_bitrate(bitrate.clone())
        .with_framerate(config.recording.framerate)
        .with_resolution(if use_scaler { Some(format!("{}x{}", target_width, target_height)) } else { None })
        .with_video_size(format!("{}x{}", width, height))
        .with_monitor_index(ddagrab_index)
        .with_audio_source(config.recording.audio_source.clone())
        .with_system_audio(system_audio_enabled)
        .with_audio_input_config(system_sample_rate, None, None, None)
        .with_audio_output_config(Some("pcm_s16le".to_string()), config.recording.audio_bitrate.clone(), DEFAULT_AUDIO_SAMPLE_RATE, DEFAULT_AUDIO_CHANNELS)
        .with_audio_backend(config.recording.audio_backend.clone())
        .with_segment_config(segment_time, wrap_limit, playlist_path);

    // 7. Spawn Session
    // 7. Spawn Session
    use crate::ffmpeg::session::RecordingSessionConfig;
    
    let session_config = RecordingSessionConfig {
        audio_source: config.recording.audio_source,
        system_audio_enabled,
        system_sample_rate,
        system_audio_device,
        audio_codec: Some("pcm_s16le".to_string()),
        audio_bitrate: config.recording.audio_bitrate,
        video_bitrate: bitrate,
        buffer_dir,
        retention_seconds: config.recording.buffer_retention_seconds,
        audio_backend: config.recording.audio_backend,
    };

    RecordingSession::spawn(
        app.clone(),
        builder,
        session_config,
    )
}

fn find_primary_monitor(window: &tauri::WebviewWindow, monitors: &[tauri::Monitor]) -> (Option<tauri::Monitor>, Option<u32>) {
    if let Ok(Some(primary)) = window.primary_monitor() {
        let primary_index = monitors.iter().position(|m| 
            m.position().x == primary.position().x && m.position().y == primary.position().y
        ).unwrap_or(0) as u32;
        
        println!("Primary Monitor found at index {}.", primary_index);
        (Some(primary), Some(primary_index))
    } else {
        println!("No Primary Monitor found. Defaulting to index 0.");
        (monitors.first().cloned(), Some(0))
    }
}
