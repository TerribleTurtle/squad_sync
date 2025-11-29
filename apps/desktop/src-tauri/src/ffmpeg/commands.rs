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
    video_size: Option<String>,
    offset_x: Option<i32>,
    offset_y: Option<i32>,
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
            video_size: None,
            offset_x: None,
            offset_y: None,
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

    pub fn with_video_size(mut self, size: String) -> Self {
        self.video_size = Some(size);
        self
    }

    pub fn with_offset(mut self, x: i32, y: i32) -> Self {
        self.offset_x = Some(x);
        self.offset_y = Some(y);
        self
    }

    pub fn build(&self) -> Vec<String> {
        let mut args = vec![
            "-f".to_string(), self.input_format.clone(),
            "-framerate".to_string(), self.framerate.to_string(),
        ];

        if let Some(size) = &self.video_size {
            args.push("-video_size".to_string());
            args.push(size.clone());
        }

        if let (Some(x), Some(y)) = (self.offset_x, self.offset_y) {
            args.push("-offset_x".to_string());
            args.push(x.to_string());
            args.push("-offset_y".to_string());
            args.push(y.to_string());
        }

        args.extend(vec![
            "-i".to_string(), self.input_source.clone(),
        ]);

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
        // No video_size or offset in default
        assert_eq!(args[4], "-i");
        assert_eq!(args[5], "desktop");
    }

    #[test]
    fn test_custom_codec() {
        let builder = FfmpegCommandBuilder::new("output.ts".to_string())
            .with_video_codec("h264_nvenc".to_string());
        let args = builder.build();
        
        // Index depends on what comes before. 
        // Default: -f gdigrab -framerate 60 -i desktop -c:v ...
        assert_eq!(args[7], "h264_nvenc");
    }

    #[test]
    fn test_region_capture() {
        let builder = FfmpegCommandBuilder::new("output.ts".to_string())
            .with_video_size("1920x1080".to_string())
            .with_offset(0, 0);
        let args = builder.build();

        assert_eq!(args[4], "-video_size");
        assert_eq!(args[5], "1920x1080");
        assert_eq!(args[6], "-offset_x");
        assert_eq!(args[7], "0");
        assert_eq!(args[8], "-offset_y");
        assert_eq!(args[9], "0");
        assert_eq!(args[10], "-i");
    }
}
