use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandEvent, CommandChild};
use crate::ffmpeg::commands::FfmpegCommandBuilder;
use crate::ffmpeg::encoder::{self, VideoEncoder};
use crate::state::RecordingState;

pub fn spawn_ffmpeg(app: &AppHandle) -> Result<CommandChild, String> {
    let app_cache = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    let state = app.state::<RecordingState>();
    let config = state.config.lock().map_err(|e| e.to_string())?;

    // Use configured path or default to buffer dir
    // Use configured path or default to buffer dir
    let (output_path_str, _is_manual) = if config.recording.mode == "manual" {
        let video_dir = app.path().video_dir().map_err(|e| e.to_string())?;
        let recordings_dir = video_dir.join("SquadSync");
        if !recordings_dir.exists() {
            std::fs::create_dir_all(&recordings_dir).map_err(|e| e.to_string())?;
        }
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let filename = format!("recording_{}.mp4", timestamp);
        let path = recordings_dir.join(filename);
        
        (path.to_string_lossy().to_string(), true)
    } else if !config.recording.path.is_empty() {
        let path = std::path::PathBuf::from(&config.recording.path).join("out_%03d.ts");
        (path.to_string_lossy().to_string(), false)
    } else {
        let buffer_dir = app_cache.join("buffer");
        if !buffer_dir.exists() {
            std::fs::create_dir_all(&buffer_dir).map_err(|e| e.to_string())?;
        }
        let path = buffer_dir.join("out_%03d.ts");
        (path.to_string_lossy().to_string(), false)
    };

    // Encoder selection
    let encoder = if config.recording.encoder == "auto" {
        encoder::get_best_encoder()
    } else {
        // Simple mapping, could be more robust
        match config.recording.encoder.as_str() {
            "h264_nvenc" => VideoEncoder::Nvenc,
            "h264_amf" => VideoEncoder::Amf,
            "h264_qsv" => VideoEncoder::Qsv,
            "h264_vaapi" => VideoEncoder::Vaapi,
            _ => VideoEncoder::X264,
        }
    };
    
    println!("Selected encoder: {:?}", encoder);

    // Get primary monitor info
    let monitor = if let Some(window) = app.get_webview_window("main") {
        window.primary_monitor().ok().flatten()
    } else {
        None
    };

    let (width, height, x, y) = if let Some(m) = monitor {
        let size = m.size();
        let pos = m.position();
        (size.width, size.height, pos.x, pos.y)
    } else {
        (1920, 1080, 0, 0) // Fallback
    };
    
    println!("Primary monitor: {}x{} at {},{}", width, height, x, y);

    println!("Primary monitor: {}x{} at {},{}", width, height, x, y);

    let preset = match encoder {
        VideoEncoder::Nvenc => "p4", // Medium (balanced quality/speed)
        VideoEncoder::Amf => "balanced",
        VideoEncoder::Qsv => "veryfast",
        VideoEncoder::Vaapi => "veryfast",
        VideoEncoder::X264 => "ultrafast",
    };

    let builder = FfmpegCommandBuilder::new(output_path_str)
        .with_video_codec(encoder.as_ffmpeg_codec().to_string())
        .with_preset(preset.to_string())
        .with_mode(config.recording.mode.clone())
        .with_capture_method(config.recording.capture_method.clone())
        .with_bitrate(config.recording.bitrate.clone())
        .with_framerate(config.recording.framerate)
        .with_resolution(config.recording.resolution.clone())
        .with_video_size(format!("{}x{}", width, height))
        .with_offset(x, y);
        
    let args = builder.build();

    println!("Spawning FFmpeg with args: {:?}", args);

    let sidecar_command = app.shell().sidecar("ffmpeg").map_err(|e| e.to_string())?;
    let (mut rx, child) = sidecar_command
        .args(args)
        .spawn()
        .map_err(|e| e.to_string())?;

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    println!("FFmpeg: {:?}", String::from_utf8(line));
                }
                CommandEvent::Stderr(line) => {
                    eprintln!("FFmpeg Error: {:?}", String::from_utf8(line));
                }
                _ => {}
            }
        }
    });
    
    Ok(child)
}
