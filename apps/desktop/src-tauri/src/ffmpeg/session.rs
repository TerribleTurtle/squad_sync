//! FFmpeg Recording Session
//! 
//! This module manages the active FFmpeg child process. It handles:
//! 1. Spawning the process with arguments from [crate::ffmpeg::commands::FfmpegCommandBuilder].
//! 2. Setting process priority.
//! 3. Monitoring output via [crate::ffmpeg::monitor::FfmpegMonitor].
//! 4. Handling graceful shutdown and cleanup.

use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use std::os::windows::process::CommandExt;
use log::{info, error, warn};

use crate::ffmpeg::commands::FfmpegCommandBuilder;
use crate::ffmpeg::monitor::FfmpegMonitor;
use crate::audio;
use crate::state::RecordingMessage;

pub struct RecordingSession {
    pub video_process: std::process::Child,
    pub audio_process: std::process::Child,
    pub video_start_time: std::time::SystemTime,
    pub audio_start_time: std::time::SystemTime,
}

impl RecordingSession {
    pub fn spawn(
        app: AppHandle,
        builder: FfmpegCommandBuilder,
        audio_source: Option<String>,
        _system_audio_enabled: bool,
        system_sample_rate: u32,
        system_audio_device: Option<String>,
        audio_codec: Option<String>,
        audio_bitrate: Option<String>,
        video_bitrate: String,
        buffer_dir: std::path::PathBuf,
        retention_seconds: u32,
        audio_backend: String,
    ) -> Result<Sender<RecordingMessage>, String> {
        let (tx, rx) = mpsc::channel::<RecordingMessage>();
        let app_clone = app.clone();

        thread::spawn(move || {
            // 1. Audio Capture Setup (Microphone)
            let (mic_stream, mic_sample_rate, mic_channels, final_audio_source) = if let Some(source) = &audio_source {
                if audio_backend == "dshow" {
                    info!("Using DShow for microphone: {}", source);
                    (None, None, None, Some(source.clone()))
                } else {
                    info!("Starting microphone capture (CPAL): {}", source);
                    match audio::start_mic_capture(Some(source.clone())) {
                        Ok((rate, channels, stream)) => (Some(stream), Some(rate), Some(channels), Some(source.clone())),
                        Err(e) => {
                            error!("Failed to start microphone capture: {}", e);
                            (None, None, None, None)
                        }
                    }
                }
            } else {
                (None, None, None, None)
            };

            // 2. System Audio Capture Setup
            let (system_stream, system_channels, system_rate, final_system_audio_enabled) = if let Some(device_name) = system_audio_device {
                info!("Starting system audio capture: {}", device_name);
                match audio::start_system_capture(Some(device_name)) {
                    Ok((rate, channels, stream)) => (Some(stream), Some(channels), Some(rate), true),
                    Err(e) => {
                        error!("Failed to start system audio capture: {}", e);
                        (None, None, None, false)
                    }
                }
            } else {
                (None, None, None, false)
            };

            // 3. Configure Audio in Builder (Base Configuration)
            let base_builder = builder
                .with_audio_source(final_audio_source)
                .with_system_audio(final_system_audio_enabled)
                .with_audio_input_config(system_rate.unwrap_or(system_sample_rate), mic_sample_rate, mic_channels, system_channels)
                .with_audio_output_config(audio_codec, audio_bitrate, crate::constants::DEFAULT_AUDIO_SAMPLE_RATE, crate::constants::DEFAULT_AUDIO_CHANNELS)
                .with_audio_backend(audio_backend.clone());

            // 4. Prepare Commands (Video & Audio)
            use crate::ffmpeg::commands::CommandMode;
            
            let video_pattern = buffer_dir.join("video_%03d.mkv").to_string_lossy().to_string();
            let audio_pattern = buffer_dir.join("audio_%03d.mkv").to_string_lossy().to_string();

            // Video Command
            let video_builder = base_builder.clone()
                .with_mode(CommandMode::VideoOnly)
                .with_output_path(video_pattern)
                .with_segment_config(
                    base_builder.get_segment_time().unwrap_or(4), 
                    base_builder.get_segment_wrap().unwrap_or(100), 
                    buffer_dir.join("video_list.m3u8").to_string_lossy().to_string()
                );

            let video_args = video_builder.build();
            
            // Audio Command
            let audio_builder = base_builder.clone()
                .with_mode(CommandMode::AudioOnly)
                .with_output_path(audio_pattern)
                .with_segment_config(
                    base_builder.get_segment_time().unwrap_or(4), 
                    base_builder.get_segment_wrap().unwrap_or(100), 
                    buffer_dir.join("audio_list.m3u8").to_string_lossy().to_string()
                );

            let audio_args = audio_builder.build();

            info!("Spawning Video Process with args: {:?}", video_args);
            info!("Spawning Audio Process with args: {:?}", audio_args);

            // 5. Spawn Processes (Video First)
            let video_sidecar = match app_clone.shell().sidecar("ffmpeg") {
                Ok(cmd) => cmd,
                Err(e) => { error!("Failed to create video sidecar: {}", e); return; }
            };
            
            let video_start_time = std::time::SystemTime::now();
            let (video_rx, mut video_child) = match video_sidecar.args(video_args).spawn() {
                Ok(res) => res,
                Err(e) => { error!("Failed to spawn Video FFmpeg: {}", e); return; }
            };

            let audio_sidecar = match app_clone.shell().sidecar("ffmpeg") {
                Ok(cmd) => cmd,
                Err(e) => { error!("Failed to create audio sidecar: {}", e); return; }
            };

            let audio_start_time = std::time::SystemTime::now();
            let (audio_rx, audio_child) = match audio_sidecar.args(audio_args).spawn() {
                Ok(res) => res,
                Err(e) => { 
                    error!("Failed to spawn Audio FFmpeg: {}", e); 
                    // Kill video if audio fails
                    let _ = video_child.write(b"q");
                    return; 
                }
            };

            let video_pid = video_child.pid();
            let audio_pid = audio_child.pid();
            
            // 5b. Write Metadata for Sync
            let metadata_path = buffer_dir.join("metadata.json");
            let v_start_ms = video_start_time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
            let a_start_ms = audio_start_time.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
            
            let metadata_json = serde_json::json!({
                "video_start_time": v_start_ms,
                "audio_start_time": a_start_ms
            });
            
            if let Ok(json_str) = serde_json::to_string_pretty(&metadata_json) {
                let _ = std::fs::write(&metadata_path, json_str);
                info!("Written sync metadata to {:?}", metadata_path);
            }

            // 6. Set High Priority (Both)
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(100));
                let _ = std::process::Command::new("powershell")
                    .args(&[
                        "-NoProfile", 
                        "-Command", 
                        &format!("Get-Process -Id {}, {} | ForEach-Object {{ $_.PriorityClass = 'High' }}", video_pid, audio_pid)
                    ])
                    .creation_flags(0x08000000)
                    .output();
                info!("Set FFmpeg PIDs {}, {} to High Priority", video_pid, audio_pid);
            });

            // 7. Start Monitor (Video & Audio)
            FfmpegMonitor::start(video_rx, Some(video_bitrate), "🔴 REC".to_string());
            FfmpegMonitor::start(audio_rx, None, "🔊 AUD".to_string());

            // 8. Event Loop
            let cleanup_interval = Duration::from_secs(30);
            let mut last_cleanup = std::time::Instant::now();

            loop {
                match rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(msg) => {
                        match msg {
                            RecordingMessage::AudioData(_) => {}
                            RecordingMessage::Stop => {
                                info!("Stopping recording...");
                                break;
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if last_cleanup.elapsed() >= cleanup_interval {
                            if let Err(e) = crate::commands::replay::cleanup_buffer(&buffer_dir, retention_seconds) {
                                error!("Background Cleanup Error: {}", e);
                            }
                            last_cleanup = std::time::Instant::now();
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        info!("Channel disconnected, stopping...");
                        break;
                    }
                }
            }

            // --- CLEANUP ---
            info!("Cleaning up FFmpeg processes...");
            drop(mic_stream);
            drop(system_stream);
            
            // GRACEFUL STOP (Parallel)
            let video_child_ptr = std::sync::Arc::new(std::sync::Mutex::new(video_child));
            let audio_child_ptr = std::sync::Arc::new(std::sync::Mutex::new(audio_child));
            
            let v_ptr = video_child_ptr.clone();
            let a_ptr = audio_child_ptr.clone();

            let t1 = thread::spawn(move || {
                if let Ok(mut child) = v_ptr.lock() {
                    info!("Sending 'q' to Video FFmpeg...");
                    if let Err(e) = child.write(b"q") {
                        warn!("Failed to send 'q' to Video: {}", e);
                    }
                }
            });

            let t2 = thread::spawn(move || {
                if let Ok(mut child) = a_ptr.lock() {
                    info!("Sending 'q' to Audio FFmpeg...");
                    if let Err(e) = child.write(b"q") {
                        warn!("Failed to send 'q' to Audio: {}", e);
                    }
                }
            });

            let _ = t1.join();
            let _ = t2.join();
            
            // Wait for exit (with timeout)
            let start = std::time::Instant::now();
            while start.elapsed().as_secs() < 5 {
                thread::sleep(Duration::from_millis(500));
            }

            // FORCE KILL (Parallel)
            info!("Ensuring FFmpeg processes are stopped...");
            let _ = std::process::Command::new("taskkill")
                .args(&["/F", "/PID", &video_pid.to_string()])
                .creation_flags(0x08000000)
                .output();
                
            let _ = std::process::Command::new("taskkill")
                .args(&["/F", "/PID", &audio_pid.to_string()])
                .creation_flags(0x08000000)
                .output();

            info!("Recording Manager Thread Exiting");
        });

        Ok(tx)
    }
}
