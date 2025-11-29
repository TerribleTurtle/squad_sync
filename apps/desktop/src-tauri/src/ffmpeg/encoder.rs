use std::process::Command;

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

pub fn get_best_encoder() -> VideoEncoder {
    let available = get_available_encoders();
    
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

fn get_available_encoders() -> Vec<VideoEncoder> {
    let mut encoders = Vec::new();
    
    // We try to run "ffmpeg -encoders" and parse the output.
    // Note: In a real Tauri app, we might need to use the sidecar here too if ffmpeg isn't in PATH.
    // However, for detection, we might want to try the sidecar first.
    // Since we are in the backend, we can try to execute the sidecar or just 'ffmpeg' if it's in path.
    // For now, let's assume we can run 'ffmpeg' or we fail back to software.
    // A more robust way would be to pass the AppHandle to this function to use the sidecar, 
    // but that complicates the signature. Let's try a simple Command first.
    
    // TODO: This relies on ffmpeg being in the PATH or accessible. 
    // If we strictly use sidecar, we might need to refactor this to be async or take AppHandle.
    // For this POC, we will try to run `ffmpeg` directly.
    
    let output = match Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-encoders")
        .output() 
    {
        Ok(o) => o,
        Err(_) => return vec![VideoEncoder::X264], // Fallback if we can't run ffmpeg
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    if stdout.contains("h264_nvenc") {
        encoders.push(VideoEncoder::Nvenc);
    }
    if stdout.contains("h264_amf") {
        encoders.push(VideoEncoder::Amf);
    }
    if stdout.contains("h264_qsv") {
        encoders.push(VideoEncoder::Qsv);
    }
    if stdout.contains("h264_vaapi") {
        encoders.push(VideoEncoder::Vaapi);
    }
    
    encoders.push(VideoEncoder::X264); // Always supported
    encoders
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
