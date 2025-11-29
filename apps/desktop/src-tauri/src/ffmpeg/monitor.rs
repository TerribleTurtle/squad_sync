use tauri_plugin_shell::process::CommandEvent;
use tokio::sync::mpsc::Receiver;

pub struct FfmpegMonitor;

impl FfmpegMonitor {
    pub fn start(mut rx: Receiver<CommandEvent>) {
        tauri::async_runtime::spawn(async move {
            let mut last_log_time = std::time::Instant::now();
            let mut first_log = true;

            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stdout(line) | CommandEvent::Stderr(line) => {
                        let line_str = String::from_utf8_lossy(&line);
                        let line_string = line_str.to_string();

                        // Check if this is a progress line
                        let is_progress = line_string.contains("frame=") && line_string.contains("time=");

                        if is_progress {
                            // Log only if 5 seconds have passed, or it's the first log
                            if first_log || last_log_time.elapsed() >= std::time::Duration::from_secs(5) {
                                // Parse and format the progress line
                                let time = extract_value(&line_string, "time=");
                                let fps = extract_value(&line_string, "fps=");
                                let bitrate = extract_value(&line_string, "bitrate=");
                                let speed = extract_value(&line_string, "speed=");
                                let dup = extract_value(&line_string, "dup=");
                                let drop = extract_value(&line_string, "drop=");
                                
                                println!(
                                    "🔴 REC | Time: {} | FPS: {} | Bitrate: {} | Speed: {} | Dup: {} | Drop: {}", 
                                    time.unwrap_or("??".to_string()), 
                                    fps.unwrap_or("??".to_string()), 
                                    bitrate.unwrap_or("??".to_string()),
                                    speed.unwrap_or("??".to_string()),
                                    dup.unwrap_or("0".to_string()),
                                    drop.unwrap_or("0".to_string())
                                );
                                
                                last_log_time = std::time::Instant::now();
                                first_log = false;
                            }
                        } else {
                            if !line_string.trim().is_empty() {
                                println!("FFmpeg: {}", line_string.trim());
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    }
}

fn extract_value(line: &str, key: &str) -> Option<String> {
    if let Some(start) = line.find(key) {
        let after_key = &line[start + key.len()..];
        // Skip leading whitespace to find the start of the value
        let value_start = after_key.find(|c: char| !c.is_whitespace()).unwrap_or(0);
        let value_part = &after_key[value_start..];
        
        // Find end of value (next whitespace or end of string)
        let end = value_part.find(|c: char| c.is_whitespace()).unwrap_or(value_part.len());
        Some(value_part[..end].to_string())
    } else {
        None
    }
}
