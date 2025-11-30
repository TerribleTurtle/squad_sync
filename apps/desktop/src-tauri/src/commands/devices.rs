use tauri::{command, AppHandle};
use cpal::traits::{DeviceTrait, HostTrait};

#[command]
pub async fn get_audio_devices(_app: AppHandle) -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let devices = host.input_devices().map_err(|e| e.to_string())?;
    let mut device_names = Vec::new();
    for device in devices {
        if let Ok(name) = device.name() {
            device_names.push(name);
        }
    }
    Ok(device_names)
}

#[command]
pub async fn get_system_audio_devices(_app: AppHandle) -> Result<Vec<String>, String> {
    let host = cpal::default_host();
    let devices = host.output_devices().map_err(|e| e.to_string())?;
    let mut device_names = Vec::new();
    for device in devices {
        if let Ok(name) = device.name() {
            device_names.push(name);
        }
    }
    Ok(device_names)
}
