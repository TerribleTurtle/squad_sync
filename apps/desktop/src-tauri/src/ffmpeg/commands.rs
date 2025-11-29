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
    mode: String,
    capture_method: String,
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
            mode: "segment".to_string(),
            capture_method: "gdigrab".to_string(),
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

    pub fn with_preset(mut self, preset: String) -> Self {
        self.preset = preset;
        self
    }

    pub fn with_mode(mut self, mode: String) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_capture_method(mut self, method: String) -> Self {
        self.capture_method = method;
        self
    }

    pub fn build(&self) -> Vec<String> {
        let mut args = Vec::new();

        if self.capture_method == "ddagrab" {
            args.push("-f".to_string());
            args.push("lavfi".to_string());
            
            // Construct ddagrab filter string
            // e.g. ddagrab=framerate=120:offset_x=0:offset_y=0:video_size=1920x1080,scale_d3d11=format=nv12
            // Optimization: Capture at 2x target framerate to reduce jitter/dups.
            // We then decimate to target framerate with -r output option.
            let capture_rate = self.framerate * 2;
            let mut filter_opts = format!("ddagrab=framerate={}", capture_rate);
            
            if let Some(size) = &self.video_size {
                filter_opts.push_str(&format!(":video_size={}", size));
            }
            
            if let (Some(x), Some(y)) = (self.offset_x, self.offset_y) {
                filter_opts.push_str(&format!(":offset_x={}:offset_y={}", x, y));
            }

            // Optimization: Keep everything on GPU. 
            // ddagrab (D3D11) -> framestep=2 (Drop every 2nd frame) -> scale_d3d11 (NV12) -> h264_nvenc
            // framestep=2 converts 120fps -> 60fps instantly without averaging.
            filter_opts.push_str(",framestep=2,scale_d3d11=format=nv12");

            args.push("-i".to_string());
            args.push(filter_opts);

        } else {
            // Default gdigrab
            args.push("-f".to_string());
            args.push("gdigrab".to_string());
            args.push("-framerate".to_string());
            args.push(self.framerate.to_string());
            
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
    
            args.push("-i".to_string());
            args.push(self.input_source.clone());
        }

        if let Some(res) = &self.resolution {
            args.push("-vf".to_string());
            args.push(format!("scale={}", res));
        }

        args.extend(vec![
            "-c:v".to_string(), self.video_codec.clone(),
            "-b:v".to_string(), self.bitrate.clone(),
        ]);

        // Use d3d11 for ddagrab (full GPU pipeline), yuv420p for gdigrab (compatibility)
        if self.capture_method == "ddagrab" {
            args.extend(vec!["-pix_fmt".to_string(), "d3d11".to_string()]);
        } else {
            args.extend(vec!["-pix_fmt".to_string(), "yuv420p".to_string()]);
        }

        args.extend(vec![
            "-preset".to_string(), self.preset.clone(),
            "-r".to_string(), self.framerate.to_string(), // Enforce CFR output
        ]);

        if self.mode == "manual" {
            args.extend(vec![
                "-f".to_string(), "mp4".to_string(),
                "-movflags".to_string(), "+faststart".to_string(),
                self.output_path.clone(),
            ]);
        } else {
            args.extend(vec![
                "-f".to_string(), "segment".to_string(),
                "-segment_time".to_string(), self.segment_time.to_string(),
                "-segment_wrap".to_string(), self.segment_wrap.to_string(),
                "-reset_timestamps".to_string(), "1".to_string(),
                self.output_path.clone(),
            ]);
        }

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

    #[test]
    fn test_manual_mode() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_mode("manual".to_string());
        let args = builder.build();

        // Should contain -f mp4 and -movflags +faststart
        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(&"mp4".to_string()));
        assert!(args.contains(&"-movflags".to_string()));
        assert!(args.contains(&"+faststart".to_string()));
        
        // Should NOT contain segment args
        assert!(!args.contains(&"segment".to_string()));
        assert!(!args.contains(&"-segment_time".to_string()));
    }

    #[test]
    fn test_ddagrab_mode() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_capture_method("ddagrab".to_string())
            .with_video_codec("h264_nvenc".to_string())
            .with_framerate(60)
            .with_video_size("1920x1080".to_string())
            .with_offset(0, 0);
        let args = builder.build();

        // Should use lavfi
        assert_eq!(args[0], "-f");
        assert_eq!(args[1], "lavfi");
        assert_eq!(args[2], "-i");
        
        // Check filter string
        // ddagrab=framerate=120:video_size=1920x1080:offset_x=0:offset_y=0,scale_d3d11=format=nv12
        let filter = &args[3];
        assert!(filter.starts_with("ddagrab="));
        assert!(filter.contains("framerate=120")); // 2x 60
        assert!(filter.contains("video_size=1920x1080"));
        assert!(filter.contains("offset_x=0"));
        assert!(filter.contains("offset_y=0"));
        assert!(filter.contains(",framestep=2,scale_d3d11=format=nv12"));
        assert!(!filter.contains("hwdownload")); // Should NOT download to CPU
        
        // Check pix_fmt
        assert!(args.contains(&"-pix_fmt".to_string()));
        assert!(args.contains(&"d3d11".to_string()));

        // Check CFR enforcement
        assert!(args.contains(&"-r".to_string()));
    }
}
