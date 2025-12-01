use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crate::constants::{SYSTEM_AUDIO_PIPE_NAME, MIC_AUDIO_PIPE_NAME, ERROR_NO_DATA, AUDIO_SILENCE_TIMEOUT_MS};
use tokio::net::windows::named_pipe::ServerOptions;
use tokio::io::AsyncWriteExt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct AudioCapture {
    pub mic_stream: Option<cpal::Stream>,
    pub system_stream: Option<cpal::Stream>,
    pub mic_pipe_task: Option<tokio::task::JoinHandle<()>>,
    pub system_pipe_task: Option<tokio::task::JoinHandle<()>>,
    pub system_sample_rate: u32,
}

pub fn start_mic_capture(device_name: Option<String>) -> Result<(u32, u16, cpal::Stream), String> {
    let host = cpal::default_host();
    let device = if let Some(name) = device_name {
        host.input_devices().map_err(|e| e.to_string())?
            .find(|d| d.name().unwrap_or_default() == name)
            .ok_or(format!("Microphone '{}' not found", name))?
    } else {
        host.default_input_device().ok_or("No input device available".to_string())?
    };

    log::info!("Microphone Device: {}", device.name().unwrap_or("Unknown".to_string()));
    create_and_start_stream(device, MIC_AUDIO_PIPE_NAME)
}

pub fn start_system_capture(device_name: Option<String>) -> Result<(u32, u16, cpal::Stream), String> {
    let host = cpal::default_host();
    let device = if let Some(name) = device_name {
        host.output_devices().map_err(|e| e.to_string())?
            .find(|d| d.name().unwrap_or_default() == name)
            .ok_or(format!("System device '{}' not found", name))?
    } else {
        host.default_output_device().ok_or("No output device available".to_string())?
    };

    log::info!("System Audio Device: {}", device.name().unwrap_or("Unknown".to_string()));
    create_and_start_stream(device, SYSTEM_AUDIO_PIPE_NAME)
}

fn create_and_start_stream(device: cpal::Device, pipe_name: &'static str) -> Result<(u32, u16, cpal::Stream), String> {
    let config = if pipe_name == SYSTEM_AUDIO_PIPE_NAME {
        device.default_output_config()
    } else {
        device.default_input_config()
    }.map_err(|e| e.to_string())?;

    let sample_rate = config.sample_rate().0;
    let channels = config.channels();
    log::info!("Audio Format for {}: {}Hz, {} channels", pipe_name, sample_rate, channels);
    
    let config: cpal::StreamConfig = config.into();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    let is_recording = Arc::new(AtomicBool::new(false));
    let is_recording_clone = is_recording.clone();

    let err_fn = move |err| log::error!("Error on stream {}: {}", pipe_name, err);

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &_| {
            if !is_recording_clone.load(Ordering::Relaxed) {
                return;
            }
            let bytes = bytemuck::cast_slice(data).to_vec();
            let _ = tx.send(bytes);
        },
        err_fn,
        None
    ).map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;

    spawn_pipe_task(pipe_name, rx, is_recording, sample_rate, channels);
    
    Ok((sample_rate, channels, stream))
}

fn spawn_pipe_task(pipe_name: &'static str, mut rx: tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>, is_recording: Arc<AtomicBool>, sample_rate: u32, channels: u16) {
    tauri::async_runtime::spawn(async move {
        let mut server = match ServerOptions::new()
            .first_pipe_instance(true)
            .create(pipe_name) {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to create named pipe {}: {}", pipe_name, e);
                    return;
                }
            };

        log::info!("Waiting for FFmpeg to connect to {}...", pipe_name);
        if let Err(e) = server.connect().await {
            log::error!("Failed to connect named pipe {}: {}", pipe_name, e);
            return;
        }
        log::info!("FFmpeg connected to {}!", pipe_name);

        is_recording.store(true, Ordering::Relaxed);

        // Silence Generation
        let silence_timeout = tokio::time::Duration::from_millis(AUDIO_SILENCE_TIMEOUT_MS);
        let mut last_write_time = std::time::Instant::now();

        loop {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(last_write_time);
            
            // Calculate remaining time until timeout
            let timeout_duration = if elapsed < silence_timeout {
                silence_timeout - elapsed
            } else {
                std::time::Duration::from_millis(0)
            };

            // Wait for data or timeout
            match tokio::time::timeout(timeout_duration, rx.recv()).await {
                Ok(Some(bytes)) => {
                    if let Err(e) = server.write_all(&bytes).await {
                        if e.kind() == std::io::ErrorKind::BrokenPipe || e.raw_os_error() == Some(ERROR_NO_DATA) {
                            log::info!("Pipe {} closed by client.", pipe_name);
                        } else {
                            log::error!("Pipe {} write error: {}", pipe_name, e);
                        }
                        return;
                    }
                    // We successfully wrote data. Update timestamp.
                    // Note: This assumes data arrived "just in time". 
                    // If we are mixing pushed data with silence, this is the best we can do without complex buffering.
                    last_write_time = std::time::Instant::now();
                }
                Ok(None) => {
                    log::info!("Audio channel closed for {}", pipe_name);
                    return;
                }
                Err(_) => {
                    // Timeout: Inject Silence based on ACTUAL elapsed time
                    let now = std::time::Instant::now();
                    let gap = now.duration_since(last_write_time);
                    
                    if gap.as_millis() > 0 {
                        // Calculate samples: (Rate * Channels * GapMS) / 1000
                        // Use u64 to prevent overflow
                        let samples_needed = (sample_rate as u64 * channels as u64 * gap.as_millis() as u64) / 1000;
                        let bytes_needed = samples_needed * 4; // f32 = 4 bytes
                        
                        if bytes_needed > 0 {
                            let silence_buf = vec![0u8; bytes_needed as usize];
                            
                            // log::debug!("Injecting {} bytes ({}ms) of silence into {}", bytes_needed, gap.as_millis(), pipe_name);
                            
                            if let Err(e) = server.write_all(&silence_buf).await {
                                if e.kind() == std::io::ErrorKind::BrokenPipe || e.raw_os_error() == Some(ERROR_NO_DATA) {
                                    log::info!("Pipe {} closed by client (during silence).", pipe_name);
                                } else {
                                    log::error!("Pipe {} write error (silence): {}", pipe_name, e);
                                }
                                return;
                            }
                        }
                        last_write_time = now;
                    }
                }
            }
        }
    });
}
