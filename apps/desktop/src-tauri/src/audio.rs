use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::Sender;
use crate::state::RecordingMessage;
use crate::constants::{SYSTEM_AUDIO_PIPE_NAME, AUDIO_LATENCY_THRESHOLD_MS};
use tokio::net::windows::named_pipe::ServerOptions;
use tokio::io::AsyncWriteExt;


pub struct AudioCapture {
    pub mic_stream: Option<cpal::Stream>,
    pub system_stream: Option<cpal::Stream>,
    pub system_pipe_task: Option<tokio::task::JoinHandle<()>>,
    pub system_sample_rate: u32,
}

pub fn start_mic_capture(device_name: Option<String>, tx: Sender<RecordingMessage>) -> Result<(u32, u16, cpal::Stream), String> {
    let host = cpal::default_host();
    
    let device = if let Some(name) = device_name {
        host.input_devices().map_err(|e| e.to_string())?
            .find(|d| d.name().unwrap_or_default() == name)
            .ok_or(format!("Microphone '{}' not found", name))?
    } else {
        host.default_input_device().ok_or("No input device available")?
    };

    println!("Microphone Device: {}", device.name().unwrap_or("Unknown".to_string()));

    let config = device.default_input_config().map_err(|e| e.to_string())?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels();
    println!("Microphone Format: {}Hz, {} channels", sample_rate, channels);
    
    let config: cpal::StreamConfig = config.into();

    let err_fn = |err| eprintln!("an error occurred on mic stream: {}", err);

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &_| {
            let mut bytes = Vec::with_capacity(data.len() * 4);
            for &sample in data {
                bytes.extend_from_slice(&sample.to_le_bytes());
            }
            let _ = tx.send(RecordingMessage::AudioData(bytes));
        },
        err_fn,
        None
    ).map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;
    
    Ok((sample_rate, channels, stream))
}

pub fn start_system_capture(device_name: Option<String>) -> Result<(u32, u16, cpal::Stream), String> {
    let host = cpal::default_host();
    
    // Find the OUTPUT device to capture from (Loopback)
    let device = if let Some(name) = device_name {
        host.output_devices().map_err(|e| e.to_string())?
            .find(|d| d.name().unwrap_or_default() == name)
            .ok_or(format!("System device '{}' not found", name))?
    } else {
        host.default_output_device().ok_or("No output device available")?
    };

    println!("System Audio Device: {}", device.name().unwrap_or("Unknown".to_string()));

    let config = device.default_output_config().map_err(|e| e.to_string())?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels();
    let config: cpal::StreamConfig = config.into();
    
    println!("System Audio Format: {}Hz, {} channels", sample_rate, channels);

    // Create a channel to send audio data from callback to pipe writer
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    let err_fn = |err| eprintln!("an error occurred on system stream: {}", err);

    // Build INPUT stream on OUTPUT device (Loopback)
    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &_| {
            let mut bytes = Vec::with_capacity(data.len() * 4);
            for &sample in data {
                bytes.extend_from_slice(&sample.to_le_bytes());
            }
            // Send to pipe writer (non-blocking)
            let _ = tx.send(bytes);
        },
        err_fn,
        None
    ).map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;

    // Spawn task to write to named pipe using Tauri's async runtime
    tauri::async_runtime::spawn(async move {
        // Create Named Pipe Server
        let mut server = match ServerOptions::new()
            .first_pipe_instance(true)
            .create(SYSTEM_AUDIO_PIPE_NAME) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to create named pipe: {}", e);
                    return;
                }
            };

        println!("Waiting for FFmpeg to connect to system audio pipe...");
        
        if let Err(e) = server.connect().await {
            eprintln!("Failed to connect named pipe: {}", e);
            return;
        }
        println!("FFmpeg connected to system audio pipe!");

        let start_time = std::time::Instant::now();
        let mut total_bytes_written: u64 = 0;
        let bytes_per_sec = sample_rate as u64 * channels as u64 * 4;
        
        // Allow a small buffer (latency) before injecting silence to avoid fighting with legitimate jitter
        let latency_threshold_bytes = (bytes_per_sec * AUDIO_LATENCY_THRESHOLD_MS) / 1000;

        loop {
            // 1. Drain all available pending packets from the channel
            while let Ok(bytes) = rx.try_recv() {
                if let Err(e) = server.write_all(&bytes).await {
                     if e.kind() == std::io::ErrorKind::BrokenPipe || e.raw_os_error() == Some(232) {
                        println!("System audio pipe closed by client.");
                    } else {
                        eprintln!("Pipe write error: {}", e);
                    }
                    return; // Exit task
                }
                total_bytes_written += bytes.len() as u64;
            }

            // 2. Check if we need to inject silence
            let elapsed = start_time.elapsed();
            let expected_bytes = (elapsed.as_millis() as u64 * bytes_per_sec) / 1000;

            if total_bytes_written + latency_threshold_bytes < expected_bytes {
                // We are falling behind (silence or underrun)
                // Calculate how much silence to inject to catch up
                let deficit = expected_bytes - total_bytes_written;
                
                // Cap the deficit to avoid huge bursts (timestamp bunching)
                // We write at most 10ms of silence at a time to keep the stream smooth
                let max_chunk_size = bytes_per_sec / 100; // 10ms
                let chunk_size = std::cmp::min(deficit, max_chunk_size);
                
                let silence_chunk = vec![0u8; chunk_size as usize];
                if let Err(e) = server.write_all(&silence_chunk).await {
                     if e.kind() == std::io::ErrorKind::BrokenPipe || e.raw_os_error() == Some(232) {
                        println!("System audio pipe closed by client (silence).");
                    } else {
                        eprintln!("Pipe write error (silence): {}", e);
                    }
                    return; // Exit task
                }
                total_bytes_written += chunk_size;
            } else {
                // We are on track or ahead, and no new data.
                // Sleep briefly to prevent busy looping
                tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            }
        }
    });

    Ok((sample_rate, channels, stream))
}
