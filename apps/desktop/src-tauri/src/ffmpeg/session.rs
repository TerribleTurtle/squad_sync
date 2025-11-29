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
        audio_codec: Option<String>,
    ) -> Result<Sender<RecordingMessage>, String> {
        let (tx, rx) = mpsc::channel::<RecordingMessage>();
        let tx_clone = tx.clone();
        let app_clone = app.clone();

        thread::spawn(move || {
            // 1. Audio Capture Setup
            let audio_capture = if audio_source.is_some() {
                Some(audio::start_audio_capture(tx_clone))
            } else {
                None
            };

            let (sample_rate, channels, audio_enabled) = match audio_capture {
                Some(Ok(ref capture)) => (capture.sample_rate, capture.channels, true),
                Some(Err(ref e)) => {
                    println!("Audio capture failed: {}", e);
                    (48000, 2, false)
                },
                None => (48000, 2, false),
            };

            // 2. Configure Audio in Builder
            if audio_enabled {
                builder = builder
                    .with_audio(audio_source, audio_codec)
                    .with_audio_config(sample_rate, channels);
            }

            // 3. Build FFmpeg Args
            let args = builder.build();
            println!("Spawning FFmpeg with args: {:?}", args);

            // 4. Spawn FFmpeg Sidecar
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
            
            // 5. Set High Priority (Crucial for GPU Scheduling)
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

            // 6. Start Monitor
            FfmpegMonitor::start(ffmpeg_rx);

            // 6. Event Loop
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
                        drop(audio_capture); // Stop audio stream
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
