use std::process::Command;
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone, PartialEq)]
pub enum VideoEncoder {
    Nvenc,
    Amf,
    Qsv,
    Vaapi,
    X264,
}

impl VideoEncoder {
    pub fn as_ffmpeg_codec(&self) -> &'static str {
        match self {
            VideoEncoder::Nvenc => "h264_nvenc",
            VideoEncoder::Amf => "h264_amf",
            VideoEncoder::Qsv => "h264_qsv",
            VideoEncoder::Vaapi => "h264_vaapi",
            VideoEncoder::X264 => "libx264",
        }
    }
}

use tauri::AppHandle;

pub fn get_best_encoder(app: &AppHandle) -> VideoEncoder {
    let available = get_available_encoders(app);
    
    // Priority list
    if available.contains(&VideoEncoder::Nvenc) {
        return VideoEncoder::Nvenc;
    }
    if available.contains(&VideoEncoder::Amf) {
        return VideoEncoder::Amf;
    }
    if available.contains(&VideoEncoder::Qsv) {
        return VideoEncoder::Qsv;
    }
    // VAAPI is often tricky on non-Linux or without specific setup, but we include it
    if available.contains(&VideoEncoder::Vaapi) {
        return VideoEncoder::Vaapi;
    }

    VideoEncoder::X264
}

fn get_available_encoders(app: &AppHandle) -> Vec<VideoEncoder> {
    let mut encoders = Vec::new();
    
    // We try to run "ffmpeg -encoders" and parse the output.
    let ffmpeg_path = crate::ffmpeg::utils::get_sidecar_path(app, "ffmpeg")
        .unwrap_or_else(|_| std::path::PathBuf::from("ffmpeg")); // Fallback to PATH if sidecar fails (unlikely)
    
    let output = match Command::new(&ffmpeg_path)
        .arg("-hide_banner")
        .arg("-encoders")
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output() 
    {
        Ok(o) => o,
        Err(_) => return vec![VideoEncoder::X264], // Fallback if we can't run ffmpeg
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Helper to check and probe
    let mut check_and_add = |name: &str, encoder: VideoEncoder| {
        if stdout.contains(name) {
            if probe_encoder(&ffmpeg_path, &encoder) {
                encoders.push(encoder);
            } else {
                log::warn!("Encoder {} is present but failed probe (driver/hardware missing?)", name);
            }
        }
    };

    check_and_add("h264_nvenc", VideoEncoder::Nvenc);
    check_and_add("h264_amf", VideoEncoder::Amf);
    check_and_add("h264_qsv", VideoEncoder::Qsv);
    check_and_add("h264_vaapi", VideoEncoder::Vaapi);
    
    encoders.push(VideoEncoder::X264); // Always supported (Software)
    encoders
}

fn probe_encoder(ffmpeg_path: &std::path::PathBuf, encoder: &VideoEncoder) -> bool {
    // Run a dummy encoding: 1 frame of black video
    // ffmpeg -y -f lavfi -i color=c=black:s=128x128 -frames:v 1 -c:v <encoder> -f null -
    
    let codec = encoder.as_ffmpeg_codec();
    
    let output = Command::new(ffmpeg_path)
        .args([
            "-y",
            "-f", "lavfi",
            "-i", "color=c=black:s=1280x720",
            "-frames:v", "1",
            "-c:v", codec,
            "-pix_fmt", "yuv420p",
            "-f", "null",
            "-"
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                true
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                log::warn!("Encoder probe failed for {}: {}", codec, stderr);
                false
            }
        },
        Err(e) => {
            log::error!("Failed to execute encoder probe for {}: {}", codec, e);
            false
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_to_codec() {
        assert_eq!(VideoEncoder::Nvenc.as_ffmpeg_codec(), "h264_nvenc");
        assert_eq!(VideoEncoder::X264.as_ffmpeg_codec(), "libx264");
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HardwareScalingMode {
    None,
    D3D11,
    CUDA,
}

pub fn get_best_scaling_mode(app: &AppHandle) -> HardwareScalingMode {
    // Check for D3D11 scaler
    if crate::ffmpeg::utils::check_filter_support(app, "scale_d3d11") {
        // Probe it to ensure it works (drivers/hardware might be flaky)
        if probe_scaler(app, "scale_d3d11") {
            return HardwareScalingMode::D3D11;
        } else {
            log::warn!("scale_d3d11 present but failed probe. Ignoring.");
        }
    }
    
    // Future: Check for CUDA scaler if needed
    
    HardwareScalingMode::None
}

fn probe_scaler(app: &AppHandle, filter_name: &str) -> bool {
    // ffmpeg -y -init_hw_device d3d11va=d3d11 -f lavfi -i color=s=64x64 -vf hwupload,scale_d3d11=w=64:h=64 -f null -
    let ffmpeg_path = crate::ffmpeg::utils::get_sidecar_path(app, "ffmpeg")
        .unwrap_or_else(|_| std::path::PathBuf::from("ffmpeg"));

    let output = Command::new(ffmpeg_path)
        .args([
            "-y",
            "-init_hw_device", "d3d11va=d3d11",
            "-f", "lavfi",
            "-i", "color=s=64x64",
            "-vf", &format!("hwupload,{}", filter_name),
            "-f", "null",
            "-"
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output();

    match output {
        Ok(o) => {
            if o.status.success() {
                true
            } else {
                let stderr = String::from_utf8_lossy(&o.stderr);
                log::warn!("Scaler probe failed for {}: {}", filter_name, stderr);
                false
            }
        },
        Err(e) => {
            log::error!("Failed to execute scaler probe for {}: {}", filter_name, e);
            false
        },
    }
}
