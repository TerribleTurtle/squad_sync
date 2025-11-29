use tauri::{command, AppHandle};
use tauri_plugin_shell::ShellExt;
use regex::Regex;

#[command]
pub async fn get_audio_devices(app: AppHandle) -> Result<Vec<String>, String> {
    let sidecar_command = app.shell().sidecar("ffmpeg").map_err(|e| e.to_string())?;
    
    let output = sidecar_command
        .args(["-list_devices", "true", "-f", "dshow", "-i", "dummy"])
        .output()
        .await
        .map_err(|e| e.to_string())?;
        
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("FFmpeg stderr: {}", stderr);
    
    let mut devices = Vec::new();
    
    // Regex to match: "Device Name" (audio)
    // We look for the quoted name on lines that end with (audio) or contain (audio)
    let device_re = Regex::new(r#""([^"]+)"\s+\(audio\)"#).map_err(|e| e.to_string())?;
    
    for line in stderr.lines() {
        if let Some(caps) = device_re.captures(line) {
            if let Some(name) = caps.get(1) {
                let device_name = name.as_str().to_string();
                println!("Found audio device: {}", device_name);
                devices.push(device_name);
            }
        }
    }
    
    println!("Total devices found: {}", devices.len());
    Ok(devices)
}
