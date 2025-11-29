#[derive(Debug, Clone)]
pub struct FfmpegCommandBuilder {
    framerate: u32,
    video_codec: String,
    bitrate: String,
    preset: String,
    output_path: String,
    resolution: Option<String>,
    video_size: Option<String>,
    monitor_index: u32,
    
    // Audio Config
    audio_source: Option<String>,
    audio_codec: Option<String>,
    audio_sample_rate: u32,
    audio_channels: u16,
}

impl FfmpegCommandBuilder {
    pub fn new(output_path: String) -> Self {
        Self {
            framerate: 60,
            video_codec: "libx264".to_string(),
            bitrate: "6M".to_string(),
            preset: "ultrafast".to_string(),
            output_path,
            resolution: None,
            video_size: None,
            monitor_index: 0,
            audio_source: None,
            audio_codec: None,
            audio_sample_rate: 48000,
            audio_channels: 2,
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

    pub fn with_preset(mut self, preset: String) -> Self {
        self.preset = preset;
        self
    }

    pub fn with_monitor_index(mut self, index: u32) -> Self {
        self.monitor_index = index;
        self
    }

    pub fn with_audio(mut self, source: Option<String>, codec: Option<String>) -> Self {
        self.audio_source = source;
        self.audio_codec = codec;
        self
    }

    pub fn with_audio_config(mut self, sample_rate: u32, channels: u16) -> Self {
        self.audio_sample_rate = sample_rate;
        self.audio_channels = channels;
        self
    }

    pub fn build(&self) -> Vec<String> {
        let mut args = Vec::new();

        // --- INPUT: DDAGRAB ---
        args.push("-f".to_string());
        args.push("lavfi".to_string());
        args.push("-thread_queue_size".to_string());
        args.push("2048".to_string());
        
        let mut filter_opts = format!("ddagrab=output_idx={}:framerate={}", self.monitor_index, self.framerate);
        if let Some(size) = &self.video_size {
            filter_opts.push_str(&format!(":video_size={}", size));
        }
        args.push("-i".to_string());
        args.push(filter_opts);

        // --- INPUT: AUDIO (Pipe) ---
        if let Some(_audio_src) = &self.audio_source {
            args.extend(vec![
                "-f".to_string(), "f32le".to_string(),
                "-ar".to_string(), self.audio_sample_rate.to_string(),
                "-ac".to_string(), self.audio_channels.to_string(),
                "-thread_queue_size".to_string(), "16384".to_string(),
                "-use_wallclock_as_timestamps".to_string(), "1".to_string(),
                "-i".to_string(), "pipe:0".to_string(),
            ]);
        }

        // --- FILTER CHAIN ---
        let mut video_filters = String::new();
        
        // Resolution Logic:
        // - None or "native" -> Native Resolution (No scaling, just format conversion)
        // - "WxH" -> Scale to WxH
        let use_native_res = match &self.resolution {
            Some(r) => r.to_lowercase() == "native",
            None => true,
        };

        if !use_native_res {
            if let Some(res) = &self.resolution {
                let parts: Vec<&str> = res.split('x').collect();
                if parts.len() == 2 {
                    video_filters.push_str(&format!("scale_d3d11={}:{}:format=nv12", parts[0], parts[1]));
                } else {
                    video_filters.push_str("scale_d3d11=format=nv12");
                }
            } else {
                video_filters.push_str("scale_d3d11=format=nv12");
            }
        } else {
            // Native: Just format conversion
            video_filters.push_str("scale_d3d11=format=nv12");
        }

        if self.audio_source.is_some() {
            let video_chain = if !video_filters.is_empty() {
                format!("[0:v]setpts=PTS-STARTPTS,{}[vout]", video_filters)
            } else {
                "[0:v]setpts=PTS-STARTPTS[vout]".to_string()
            };
            let audio_chain = "[1:a]aresample=async=1,asetpts=PTS-STARTPTS[aout]";
            args.extend(vec![
                "-filter_complex".to_string(), format!("{};{}", audio_chain, video_chain),
                "-map".to_string(), "[vout]".to_string(),
                "-map".to_string(), "[aout]".to_string(),
            ]);
        } else {
            if !video_filters.is_empty() {
                args.push("-vf".to_string());
                args.push(video_filters);
            }
            args.extend(vec!["-map".to_string(), "0:v".to_string()]);
        }

        // --- ENCODING ---
        args.extend(vec!["-c:v".to_string(), self.video_codec.clone()]);
        if self.video_codec.contains("nvenc") {
            args.extend(vec![
                "-rc".to_string(), "vbr".to_string(),
                "-b:v".to_string(), self.bitrate.clone(),   // Use configured bitrate
                // Dynamic Maxrate/Bufsize based on target bitrate
                // We parse the bitrate string (e.g. "30M") to calculate these
                "-maxrate".to_string(), format!("{}M", self.bitrate.replace("M", "").parse::<u32>().unwrap_or(8) * 3 / 2), // 1.5x target
                "-bufsize".to_string(), format!("{}M", self.bitrate.replace("M", "").parse::<u32>().unwrap_or(8) * 2),     // 2.0x target
                "-preset".to_string(), "p4".to_string(), // Upgraded to p4 (Medium) for better quality
                "-tune".to_string(), "ull".to_string(),  // Re-enabled ull for speed
                "-profile:v".to_string(), "high".to_string(),
            ]);
        } else if self.video_codec.contains("amf") {
            // AMD AMF Specifics
            args.extend(vec![
                "-rc".to_string(), "cbr".to_string(), // AMF often prefers CBR for stability
                "-b:v".to_string(), self.bitrate.clone(),
                "-usage".to_string(), "transcoding".to_string(), // Real-time optimization
                "-quality".to_string(), "speed".to_string(),     // Prioritize speed
                "-profile:v".to_string(), "high".to_string(),
            ]);
        } else if self.video_codec.contains("qsv") {
            // Intel QSV Specifics
            args.extend(vec![
                "-rc".to_string(), "vbr".to_string(),
                "-b:v".to_string(), self.bitrate.clone(),
                "-preset".to_string(), "veryfast".to_string(),   // Speed priority
                "-profile:v".to_string(), "high".to_string(),
            ]);
        } else {
            // Software (CPU) Fallback - libx264
            // CRITICAL: Must be ultrafast to have any chance of real-time 1080p+
            args.extend(vec![
                "-b:v".to_string(), self.bitrate.clone(),
                "-preset".to_string(), "ultrafast".to_string(),
                "-tune".to_string(), "zerolatency".to_string(),
            ]);
        }

        // --- FRAMERATE ---
        args.extend(vec![
            "-fps_mode".to_string(), "cfr".to_string(),
            "-r".to_string(), "60".to_string(),
        ]);

        // --- GOP / KEYFRAMES ---
        let gop = self.framerate * 2;
        args.extend(vec![
            "-g".to_string(), gop.to_string(),
            "-bf".to_string(), "0".to_string(), // Disabled B-frames for speed
        ]);

        // --- AUDIO ENCODING ---
        if self.audio_source.is_some() {
            args.extend(vec![
                "-c:a".to_string(), self.audio_codec.clone().unwrap_or_else(|| "aac".to_string()),
                "-b:a".to_string(), "192k".to_string(),
            ]);
        }

        // --- OUTPUT ---
        args.extend(vec![
            "-movflags".to_string(), "+frag_keyframe+empty_moov+default_base_moof".to_string(), // Safer MP4
            "-max_muxing_queue_size".to_string(), "9999".to_string(),
            "-stats".to_string(),
            "-shortest".to_string(),
            "-f".to_string(), "mp4".to_string(),
            self.output_path.clone(),
        ]);

        // println!("Final FFmpeg args: {:?}", args);
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string());
        let args = builder.build();
        
        // Basic checks
        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(&"lavfi".to_string()));
        assert!(args.contains(&"output.mp4".to_string()));
        assert!(args.contains(&"libx264".to_string())); // Default codec
    }

    #[test]
    fn test_builder_nvenc() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_video_codec("h264_nvenc".to_string());
        let args = builder.build();
        
        assert!(args.contains(&"h264_nvenc".to_string()));
        assert!(args.contains(&"-tune".to_string()));
        assert!(args.contains(&"ull".to_string())); // NVENC specific
    }

    #[test]
    fn test_builder_audio() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_audio(Some("Mic".to_string()), Some("aac".to_string()));
        let args = builder.build();
        
        assert!(args.contains(&"pipe:0".to_string()));
        assert!(args.contains(&"aac".to_string()));
        // Check for filter complex
        let has_filter = args.iter().any(|a| a.contains("aresample"));
        assert!(has_filter);
    }
}
