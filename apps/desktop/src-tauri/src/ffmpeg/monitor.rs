use tauri_plugin_shell::process::CommandEvent;
use tokio::sync::mpsc::Receiver;

pub struct FfmpegMonitor;

impl FfmpegMonitor {
    pub fn start(mut rx: Receiver<CommandEvent>, target_bitrate: Option<String>, label: String) {
        tauri::async_runtime::spawn(async move {
            let mut last_log_time = std::time::Instant::now();
            let mut first_log = true;

            while let Some(event) = rx.recv().await {
                match event {
                    CommandEvent::Stdout(line) | CommandEvent::Stderr(line) => {
                        let line_str = String::from_utf8_lossy(&line);
                        let line_string = line_str.to_string();

                        // Check if this is a progress line
                        // Video has "frame=", Audio usually has "size=" but no "frame="
                        // Both have "time=" and "bitrate="
                        let is_progress = line_string.contains("time=") && line_string.contains("bitrate=");

                        if is_progress {
                            // Parse and format the progress line
                            let frame = extract_value(&line_string, "frame=");
                            let fps = extract_value(&line_string, "fps=");
                            
                            // For video, skip if frame is 0 or fps is 0.0 (initial warmup)
                            if let (Some(f), Some(fp)) = (&frame, &fps) {
                                if f == "0" || fp == "0.0" {
                                    continue;
                                }
                            }

                            // Log only if 5 seconds have passed, or it's the first log
                            if first_log || last_log_time.elapsed() >= std::time::Duration::from_secs(5) {
                                let time = extract_value(&line_string, "time=");
                                let mut bitrate = extract_value(&line_string, "bitrate=");
                                let speed = extract_value(&line_string, "speed=");
                                let dup = extract_value(&line_string, "dup=");
                                let drop = extract_value(&line_string, "drop=");

                                // Fallback to target bitrate if N/A
                                if bitrate.as_deref() == Some("N/A") || bitrate.is_none() {
                                    if let Some(target) = &target_bitrate {
                                        bitrate = Some(format!("{} (Target)", target));
                                    }
                                }
                                
                                // Construct log message based on available fields
                                let mut log_msg = format!(
                                    "{} | Time: {} | Bitrate: {} | Speed: {}", 
                                    label,
                                    time.unwrap_or("??".to_string()), 
                                    bitrate.unwrap_or("N/A".to_string()),
                                    speed.unwrap_or("??".to_string())
                                );

                                // Add Video-specific fields if present
                                if let Some(f) = fps {
                                    log_msg.push_str(&format!(" | FPS: {}", f));
                                }
                                if let Some(d) = dup {
                                    log_msg.push_str(&format!(" | Dup: {}", d));
                                }
                                if let Some(d) = drop {
                                    log_msg.push_str(&format!(" | Drop: {}", d));
                                }

                                log::info!("{}", log_msg);
                                
                                last_log_time = std::time::Instant::now();
                                first_log = false;
                            }
                        } else {
                            if !line_string.trim().is_empty() {
                                log::debug!("FFmpeg ({}): {}", label, line_string.trim());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_value() {
        let line = "frame= 123 fps= 60.0 size= 1024kB time=00:00:10.00 bitrate= 2000.0kbits/s speed= 1.0x";
        
        assert_eq!(extract_value(line, "frame="), Some("123".to_string()));
        assert_eq!(extract_value(line, "fps="), Some("60.0".to_string()));
        assert_eq!(extract_value(line, "time="), Some("00:00:10.00".to_string()));
        assert_eq!(extract_value(line, "bitrate="), Some("2000.0kbits/s".to_string()));
        assert_eq!(extract_value(line, "speed="), Some("1.0x".to_string()));
        assert_eq!(extract_value(line, "missing="), None);
    }

    #[test]
    fn test_is_progress_check() {
        // Video line
        let video_line = "frame= 123 fps= 60.0 size= 1024kB time=00:00:10.00 bitrate= 2000.0kbits/s speed= 1.0x";
        assert!(video_line.contains("time=") && video_line.contains("bitrate="));

        // Audio line (no frame)
        let audio_line = "size= 512kB time=00:00:30.00 bitrate= 128.0kbits/s speed= 1.0x";
        assert!(audio_line.contains("time=") && audio_line.contains("bitrate="));

        // Random line
        let random_line = "Input #0, matroska,webm, from 'input.mkv':";
        assert!(!(random_line.contains("time=") && random_line.contains("bitrate=")));
    }
}
