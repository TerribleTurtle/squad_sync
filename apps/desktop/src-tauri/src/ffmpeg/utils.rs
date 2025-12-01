//! Utility functions for FFmpeg operations.
use std::path::PathBuf;
use std::process::Command;


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
    // 0.1 bits per pixel
    let bits_per_sec = (pixels * fps_u64) / 10;
    format!("{}k", bits_per_sec / 1000)
}

/// Gets the duration of a media file in seconds using ffprobe.
/// 
/// # Arguments
/// * `path` - Path to the media file.
/// 
/// # Returns
/// * `Result<f64, String>` - The duration in seconds, or an error message.
pub fn get_file_duration(path: &PathBuf) -> Result<f64, String> {
    // ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 input.mp4
    let output = Command::new("ffprobe")
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
        // Bits/sec = 2,073,600 * 60 / 10 = 12,441,600
        // kbps = 12,441
        assert_eq!(calculate_dynamic_bitrate(1920, 1080, 60), "12441k");
    }
}
