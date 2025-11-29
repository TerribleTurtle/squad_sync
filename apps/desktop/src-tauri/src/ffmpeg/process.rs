use tauri::{AppHandle, Manager};
use crate::ffmpeg::commands::FfmpegCommandBuilder;
use crate::ffmpeg::encoder::{self, VideoEncoder};
use crate::ffmpeg::session::RecordingSession;
use crate::state::RecordingState;
use crate::state::RecordingMessage;
use std::sync::mpsc::Sender;

pub fn start_recording_process(app: &AppHandle) -> Result<Sender<RecordingMessage>, String> {
    let _app_cache = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    let state = app.state::<RecordingState>();
    let config = state.config.lock().map_err(|e| e.to_string())?.clone(); 

    // 1. Determine Output Path
    let output_path_str = if !config.recording.path.is_empty() {
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let filename = format!("recording_{}.mp4", timestamp);
        std::path::PathBuf::from(&config.recording.path).join(filename).to_string_lossy().to_string()
    } else {
        let video_dir = app.path().video_dir().map_err(|e| e.to_string())?;
        let recordings_dir = video_dir.join("SquadSync");
        if !recordings_dir.exists() {
            std::fs::create_dir_all(&recordings_dir).map_err(|e| e.to_string())?;
        }
        
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let filename = format!("recording_{}.mp4", timestamp);
        recordings_dir.join(filename).to_string_lossy().to_string()
    };

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
        window.primary_monitor().ok().flatten()
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

    let preset = match encoder {
        VideoEncoder::Nvenc => "p1", 
        VideoEncoder::Amf => "balanced",
        VideoEncoder::Qsv => "veryfast",
        VideoEncoder::Vaapi => "veryfast",
        VideoEncoder::X264 => "ultrafast",
    };

    // 4. Build Command
    let builder = FfmpegCommandBuilder::new(output_path_str)
        .with_video_codec(encoder.as_ffmpeg_codec().to_string())
        .with_preset(preset.to_string())
        .with_bitrate(config.recording.bitrate.clone())
        .with_framerate(config.recording.framerate)
        .with_resolution(config.recording.resolution.clone())
        .with_video_size(format!("{}x{}", width, height))
        .with_monitor_index(0);

    // 5. Spawn Session
    RecordingSession::spawn(
        app.clone(),
        builder,
        config.recording.audio_source,
        config.recording.audio_codec,
    )
}
