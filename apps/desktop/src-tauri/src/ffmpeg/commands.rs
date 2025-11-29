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
    resolution: Option<String>,
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
            resolution: None,
        }
    }

    pub fn with_video_codec(mut self, codec: String) -> Self {
        self.video_codec = codec;
        self
    }

    pub fn with_bitrate(mut self, bitrate: String) -> Self {
        self.bitrate = bitrate;
        self
    }

    pub fn with_framerate(mut self, framerate: u32) -> Self {
        self.framerate = framerate;
        self
    }

    pub fn with_resolution(mut self, resolution: Option<String>) -> Self {
        self.resolution = resolution;
        self
    }

    pub fn build(&self) -> Vec<String> {
        let mut args = vec![
            "-f".to_string(), self.input_format.clone(),
            "-framerate".to_string(), self.framerate.to_string(),
            "-i".to_string(), self.input_source.clone(),
        ];

        if let Some(res) = &self.resolution {
            args.push("-vf".to_string());
            args.push(format!("scale={}", res));
        }

        args.extend(vec![
            "-c:v".to_string(), self.video_codec.clone(),
            "-b:v".to_string(), self.bitrate.clone(),
            "-pix_fmt".to_string(), "yuv420p".to_string(),
            "-preset".to_string(), self.preset.clone(),
            "-f".to_string(), "segment".to_string(),
            "-segment_time".to_string(), self.segment_time.to_string(),
            "-segment_wrap".to_string(), self.segment_wrap.to_string(),
            "-reset_timestamps".to_string(), "1".to_string(),
            self.output_path.clone(),
        ]);

        args
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
        assert_eq!(args[16], "-segment_time");
        assert_eq!(args[19], "70");
        assert_eq!(args.last().unwrap(), "output.ts");
    }

    #[test]
    fn test_custom_codec() {
        let builder = FfmpegCommandBuilder::new("output.ts".to_string())
            .with_video_codec("h264_nvenc".to_string());
        let args = builder.build();
        
        assert_eq!(args[7], "h264_nvenc");
    }
}
