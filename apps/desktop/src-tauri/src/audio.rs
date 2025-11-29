use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::Sender;
use crate::state::RecordingMessage;

pub struct AudioCapture {
    pub stream: cpal::Stream,
    pub sample_rate: u32,
    pub channels: u16,
}

pub fn start_audio_capture(tx: Sender<RecordingMessage>) -> Result<AudioCapture, String> {
    let host = cpal::default_host();
    let device = host.default_input_device().ok_or("No input device available")?;
    
    println!("Audio Input Device: {}", device.name().unwrap_or("Unknown".to_string()));

    // Try to find a config that supports f32
    let supported_configs = device.supported_input_configs().map_err(|e| e.to_string())?;
    let supported_config = supported_configs
        .filter(|c| c.sample_format() == cpal::SampleFormat::F32)
        .next()
        .ok_or("Device does not support F32 sample format")?;

    // Use a standard config if possible, otherwise use the supported range's max
    let config = supported_config.with_max_sample_rate();
    let sample_rate = config.sample_rate().0;
    let channels = config.channels();
    let config: cpal::StreamConfig = config.into();

    println!("Audio Config: {}Hz, {} channels", sample_rate, channels);

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &_| {
            // Convert f32 samples to raw bytes
            let mut bytes = Vec::with_capacity(data.len() * 4);
            for &sample in data {
                bytes.extend_from_slice(&sample.to_le_bytes());
            }
            // Send to channel (ignore errors if receiver is closed)
            let _ = tx.send(RecordingMessage::AudioData(bytes));
        },
        err_fn,
        None
    ).map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;
    
    Ok(AudioCapture {
        stream,
        sample_rate,
        channels,
    })
}
