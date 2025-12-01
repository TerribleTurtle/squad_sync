//! Utility functions for FFmpeg operations.
use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Manager};

/// Resolves the path to a sidecar executable (ffmpeg/ffprobe).
/// Handles the target triple suffix automatically.
pub fn get_sidecar_path(app: &AppHandle, tool_name: &str) -> Result<PathBuf, String> {
    let target_triple = tauri::utils::platform::target_triple().map_err(|e| e.to_string())?;
    let binary_name = format!("{}-{}{}", tool_name, target_triple, if cfg!(windows) { ".exe" } else { "" });
    let short_name = format!("{}{}", tool_name, if cfg!(windows) { ".exe" } else { "" });
    
    let mut checked_paths = Vec::new();

    // 1. Try Resource Dir (Production)
    // Tauri bundles externalBin to the root of the resources directory (usually).
    // But if we defined it as "bin/ffmpeg", it might be in "bin/"?
    // Let's check both, prioritizing the root as that's standard Tauri behavior.
    if let Ok(resource_dir) = app.path().resource_dir() {
        let candidates = vec![
            resource_dir.join(&binary_name),           // resources/ffmpeg-target.exe (Standard)
            resource_dir.join("bin").join(&binary_name), // resources/bin/ffmpeg-target.exe (If preserved)
            resource_dir.join(&short_name),            // resources/ffmpeg.exe (Renamed)
            resource_dir.join("bin").join(&short_name), // resources/bin/ffmpeg.exe
        ];
        
        for path in candidates {
            if path.exists() {
                log::info!("Found sidecar at (prod): {:?}", path);
                return Ok(path);
            }
            checked_paths.push(path);
        }
    }
    
    // 2. Try Dev Path (Relative to CWD)
    // We moved binaries to src-tauri/bin, so that is the PRIMARY location.
    if let Ok(cwd) = std::env::current_dir() {
        let dev_candidates = vec![
            // If CWD is apps/desktop
            cwd.join("src-tauri").join("bin").join(&binary_name),
            // If CWD is src-tauri
            cwd.join("bin").join(&binary_name),
            // Fallbacks (Root) - Low priority
            cwd.join("src-tauri").join(&binary_name),
            cwd.join(&binary_name),
        ];

        for path in dev_candidates {
            if path.exists() {
                log::info!("Found sidecar at (dev): {:?}", path);
                return Ok(path);
            }
            checked_paths.push(path);
        }
    }
    
    // 3. Fallback: Just try the command name (System PATH)
    // Only if explicitly allowed or if we really can't find the bundled one.
    // This is risky if the system one is different version, but better than crashing?
    // User requested update, maybe they want to DISABLE this? 
    // No, fallback is usually good. But let's log it clearly.
    if which::which(tool_name).is_ok() {
        log::warn!("Sidecar not found in bundle. Falling back to system PATH for '{}'", tool_name);
        return Ok(PathBuf::from(tool_name));
    }
    
    Err(format!("Failed to find sidecar '{}'. Checked: {:?}", tool_name, checked_paths))
}


/// Parses a bitrate string (e.g., "6M", "5000k") into bits per second.
/// 
/// # Arguments
/// * `s` - The bitrate string to parse.
/// 
/// # Returns
/// * `u32` - The bitrate in bits per second.
pub fn parse_bitrate(s: &str) -> u32 {
    if s.ends_with('M') {
        s.trim_end_matches('M').parse::<u32>().unwrap_or(8) * 1_000_000
    } else if s.ends_with('k') {
        s.trim_end_matches('k').parse::<u32>().unwrap_or(8000) * 1000
    } else {
        s.parse::<u32>().unwrap_or(8000)
    }
}

/// Calculates a dynamic bitrate based on resolution and framerate.
/// Uses a heuristic of 0.1 bits per pixel.
/// 
/// # Arguments
/// * `width` - Video width in pixels.
/// * `height` - Video height in pixels.
/// * `fps` - Video framerate.
/// 
/// # Returns
/// * `String` - The calculated bitrate string (e.g., "12000k").
pub fn calculate_dynamic_bitrate(width: u32, height: u32, fps: u32) -> String {
    let pixels = width as u64 * height as u64;
    let fps_u64 = fps as u64;
    // 0.16 bits per pixel (Higher quality for gaming)
    let bits_per_sec = (pixels * fps_u64) / 6;
    format!("{}k", bits_per_sec / 1000)
}

/// Gets the duration of a media file in seconds using ffprobe.
/// 
/// # Arguments
/// * `path` - Path to the media file.
/// 
/// # Returns
/// * `Result<f64, String>` - The duration in seconds, or an error message.
pub fn get_file_duration(app: &AppHandle, path: &PathBuf) -> Result<f64, String> {
    // ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 input.mp4
    let ffprobe_path = get_sidecar_path(app, "ffprobe")
        .map_err(|e| format!("FFprobe not found: {}", e))?;

    let output = Command::new(ffprobe_path)
        .arg("-v").arg("error")
        .arg("-show_entries").arg("format=duration")
        .arg("-of").arg("default=noprint_wrappers=1:nokey=1")
        .arg(path)
        .output()
        .map_err(|e| format!("Failed to execute ffprobe: {}", e))?;

    if !output.status.success() {
        return Err(format!("ffprobe failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let duration_str = stdout.trim();
    
    duration_str.parse::<f64>()
        .map_err(|e| format!("Failed to parse duration '{}': {}", duration_str, e))
}

/// Checks if a specific FFmpeg filter is available.
pub fn check_filter_support(app: &AppHandle, filter_name: &str) -> bool {
    if let Ok(ffmpeg_path) = get_sidecar_path(app, "ffmpeg") {
        // Run `ffmpeg -filters`
        // Output format: " ... scale_d3d11      V->V       Resize video using Direct3D 11."
        if let Ok(output) = Command::new(ffmpeg_path)
            .arg("-filters")
            .output() 
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Relaxed check: Just look for the name. scale_d3d11 is unique enough.
                let found = stdout.contains(filter_name);
                
                if !found {
                    log::warn!("Filter '{}' not found in FFmpeg.", filter_name);
                    // Debug: Log all filters that look like "scale"
                    let scales: Vec<&str> = stdout.lines()
                        .filter(|l| l.contains("scale"))
                        .collect();
                    log::debug!("Available 'scale' filters: {:?}", scales);
                } else {
                    log::info!("Filter '{}' found.", filter_name);
                }

                return found;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bitrate() {
        assert_eq!(parse_bitrate("6M"), 6_000_000);
        assert_eq!(parse_bitrate("5000k"), 5_000_000);
        assert_eq!(parse_bitrate("8000"), 8000);
        assert_eq!(parse_bitrate("invalid"), 8000); // Default
    }

    #[test]
    fn test_calculate_dynamic_bitrate() {
        // 1920x1080 @ 60fps
        // Pixels = 2,073,600
        // Bits/sec = 2,073,600 * 60 / 6 = 20,736,000
        // kbps = 20,736
        assert_eq!(calculate_dynamic_bitrate(1920, 1080, 60), "20736k");
    }
}
