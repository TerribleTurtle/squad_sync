use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use std::os::windows::process::CommandExt;

use crate::ffmpeg::commands::FfmpegCommandBuilder;
use crate::ffmpeg::monitor::FfmpegMonitor;
use crate::audio;
use crate::state::RecordingMessage;

pub struct RecordingSession;

impl RecordingSession {
    pub fn spawn(
        app: AppHandle,
        mut builder: FfmpegCommandBuilder,
        audio_source: Option<String>,
        system_audio_enabled: bool,
        system_sample_rate: u32,
        system_audio_device: Option<String>,
        audio_codec: Option<String>,
        audio_bitrate: Option<String>,
    ) -> Result<Sender<RecordingMessage>, String> {
        let (tx, rx) = mpsc::channel::<RecordingMessage>();
        let tx_clone = tx.clone();
        let app_clone = app.clone();

        thread::spawn(move || {
            // 1. Audio Capture Setup (Microphone)
            let (mic_stream, mic_sample_rate, mic_channels) = if let Some(source) = &audio_source {
                println!("Starting microphone capture: {}", source);
                match audio::start_mic_capture(Some(source.clone()), tx_clone) {
                    Ok((rate, channels, stream)) => (Some(stream), Some(rate), Some(channels)),
                    Err(e) => {
                        eprintln!("Failed to start microphone capture: {}", e);
                        // Decide: Fail hard or continue without mic?
                        // For now, let's continue but log it.
                        (None, None, None)
                    }
                }
            } else {
                (None, None, None)
            };

            // 2. System Audio Capture Setup
            let (system_stream, system_channels) = if let Some(device_name) = system_audio_device {
                println!("Starting system audio capture: {}", device_name);
                // Note: start_system_capture spawns a background task for pipe writing
                // We just hold the stream here to keep it alive
                match audio::start_system_capture(Some(device_name)) {
                    Ok((_rate, channels, stream)) => (Some(stream), Some(channels)),
                    Err(e) => {
                        eprintln!("Failed to start system audio capture: {}", e);
                        (None, None)
                    }
                }
            } else {
                (None, None)
            };

            // 3. Configure Audio in Builder
            builder = builder.with_audio(
                audio_source, 
                system_audio_enabled, 
                system_sample_rate, 
                mic_sample_rate,
                mic_channels,
                system_channels,
                audio_codec, 
                audio_bitrate
            );

            // 4. Build FFmpeg Args
            let args = builder.build();
            println!("Spawning FFmpeg with args: {:?}", args);

            // 5. Spawn FFmpeg Sidecar
            let sidecar_command = match app_clone.shell().sidecar("ffmpeg") {
                Ok(cmd) => cmd,
                Err(e) => {
                    eprintln!("Failed to create sidecar command: {}", e);
                    return;
                }
            };

            let (ffmpeg_rx, mut child) = match sidecar_command.args(args).spawn() {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("Failed to spawn FFmpeg: {}", e);
                    return;
                }
            };

            let pid = child.pid();
            
            // 6. Set High Priority (Crucial for GPU Scheduling)
            let pid_u32 = pid;
            thread::spawn(move || {
                // Give it a tiny moment to initialize
                thread::sleep(Duration::from_millis(100));
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                let _ = std::process::Command::new("powershell")
                    .args(&[
                        "-NoProfile", 
                        "-Command", 
                        &format!("Get-Process -Id {} | ForEach-Object {{ $_.PriorityClass = 'High' }}", pid_u32)
                    ])
                    .creation_flags(CREATE_NO_WINDOW) // Hide the window
                    .output();
                println!("Set FFmpeg (PID: {}) to High Priority", pid_u32);
            });

            // 7. Start Monitor
            FfmpegMonitor::start(ffmpeg_rx);

            // 8. Event Loop
            while let Ok(msg) = rx.recv() {
                match msg {
                    RecordingMessage::AudioData(bytes) => {
                        if let Err(e) = child.write(&bytes) {
                            eprintln!("Failed to write audio to FFmpeg: {}", e);
                            break; 
                        }
                    }
                    RecordingMessage::Stop => {
                        println!("Stopping recording...");
                        drop(mic_stream);    // Stop mic stream
                        drop(system_stream); // Stop system stream
                        drop(child);         // Close stdin to trigger -shortest
                        
                        // Safety Net: Force kill
                        let pid = pid;
                        thread::spawn(move || {
                            thread::sleep(Duration::from_secs(2));
                            const CREATE_NO_WINDOW: u32 = 0x08000000;
                            let _ = std::process::Command::new("taskkill")
                                .args(&["/F", "/PID", &pid.to_string()])
                                .creation_flags(CREATE_NO_WINDOW) // Hide the window
                                .output();
                        });

                        break;
                    }
                }
            }
            println!("Recording Manager Thread Exiting");
        });

        Ok(tx)
    }
}
