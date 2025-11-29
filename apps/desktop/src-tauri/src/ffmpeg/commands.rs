#[derive(Debug, Clone)]
pub struct FfmpegCommandBuilder {
    input_format: String,
    framerate: u32,
    input_source: String,
    video_codec: String,
    bitrate: String,
    preset: String,
    segment_time: u32,
    segment_wrap: u32,
    output_path: String,
}

impl FfmpegCommandBuilder {
    pub fn new(output_path: String) -> Self {
        Self {
            input_format: "gdigrab".to_string(),
            framerate: 60,
            input_source: "desktop".to_string(),
            video_codec: "libx264".to_string(), // Default to software for compatibility
            bitrate: "6M".to_string(),
            preset: "ultrafast".to_string(),
            segment_time: 1,
            segment_wrap: 70,
            output_path,
        }
    }

    pub fn build(&self) -> Vec<String> {
        vec![
            "-f".to_string(), self.input_format.clone(),
            "-framerate".to_string(), self.framerate.to_string(),
            "-i".to_string(), self.input_source.clone(),
            "-c:v".to_string(), self.video_codec.clone(),
            "-b:v".to_string(), self.bitrate.clone(),
            "-preset".to_string(), self.preset.clone(),
            "-f".to_string(), "segment".to_string(),
            "-segment_time".to_string(), self.segment_time.to_string(),
            "-segment_wrap".to_string(), self.segment_wrap.to_string(),
            "-reset_timestamps".to_string(), "1".to_string(),
            self.output_path.clone(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_builder() {
        let builder = FfmpegCommandBuilder::new("output.ts".to_string());
        let args = builder.build();
        
        assert_eq!(args[0], "-f");
        assert_eq!(args[1], "gdigrab");
        assert_eq!(args[2], "-framerate");
        assert_eq!(args[3], "60");
        assert_eq!(args[14], "-segment_time");
        assert_eq!(args[16], "-segment_wrap");
        assert_eq!(args[17], "70");
        assert_eq!(args.last().unwrap(), "output.ts");
    }
}
