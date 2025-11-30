#[derive(Debug, Clone)]
pub struct FfmpegCommandBuilder {
    framerate: u32,
    video_codec: String,
    bitrate: String,
    preset: Option<String>,
    tune: Option<String>,
    profile: Option<String>,
    output_path: String,
    resolution: Option<String>,
    video_size: Option<String>,
    monitor_index: u32,
    
    // Audio Config
    audio_source: Option<String>, // Microphone
    system_audio: bool,           // System Audio Enabled
    system_sample_rate: u32,      // Detected System Sample Rate
    mic_sample_rate: Option<u32>, // Detected Mic Sample Rate
    mic_channels: Option<u16>,    // Detected Mic Channels
    system_channels: Option<u16>, // Detected System Channels
    audio_codec: Option<String>,
    audio_bitrate: Option<String>,
    audio_sample_rate: u32,       // Target Sample Rate (e.g. 48000)
    audio_channels: u16,

    // Segment Config
    segment_time: Option<u32>,
    segment_wrap: Option<u32>,
    segment_list: Option<String>,
}

use crate::constants::{SYSTEM_AUDIO_PIPE_NAME, FFMPEG_THREAD_QUEUE_SIZE, FFMPEG_EXTRA_HW_FRAMES};

impl FfmpegCommandBuilder {
    pub fn new(output_path: String) -> Self {
        Self {
            framerate: 60,
            video_codec: "libx264".to_string(),
            bitrate: "6M".to_string(),
            preset: None,
            tune: None,
            profile: None,
            output_path,
            resolution: None,
            video_size: None,
            monitor_index: 0,
            audio_source: None,
            system_audio: false,
            system_sample_rate: 48000,
            mic_sample_rate: None,
            mic_channels: None,
            system_channels: None,
            audio_codec: None,
            audio_bitrate: None,
            audio_sample_rate: 48000,
            audio_channels: 2,
            segment_time: None,
            segment_wrap: None,
            segment_list: None,
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

    pub fn with_preset(mut self, preset: Option<String>) -> Self {
        self.preset = preset;
        self
    }

    pub fn with_tune(mut self, tune: Option<String>) -> Self {
        self.tune = tune;
        self
    }

    pub fn with_profile(mut self, profile: Option<String>) -> Self {
        self.profile = profile;
        self
    }

    pub fn with_monitor_index(mut self, index: u32) -> Self {
        self.monitor_index = index;
        self
    }

    pub fn with_audio(mut self, source: Option<String>, system_audio: bool, system_rate: u32, mic_rate: Option<u32>, mic_channels: Option<u16>, system_channels: Option<u16>, codec: Option<String>, bitrate: Option<String>) -> Self {
        self.audio_source = source;
        self.system_audio = system_audio;
        self.system_sample_rate = system_rate;
        self.mic_sample_rate = mic_rate;
        self.mic_channels = mic_channels;
        self.system_channels = system_channels;
        self.audio_codec = codec;
        self.audio_bitrate = bitrate;
        self
    }

    pub fn with_audio_config(mut self, sample_rate: u32, channels: u16) -> Self {
        self.audio_sample_rate = sample_rate;
        self.audio_channels = channels;
        self
    }

    pub fn with_segment_config(mut self, time: u32, wrap: u32, list: String) -> Self {
        self.segment_time = Some(time);
        self.segment_wrap = Some(wrap);
        self.segment_list = Some(list);
        self
    }

    pub fn build(&self) -> Vec<String> {
        let mut args = Vec::new();

        // --- INPUTS ---
        
        // Input 0: Video (ddagrab)
        args.push("-f".to_string());
        args.push("lavfi".to_string());
        args.push("-thread_queue_size".to_string());
        args.push(FFMPEG_THREAD_QUEUE_SIZE.to_string());
        
        // Restore correct video_size logic to fix cropping
        let mut filter_opts = format!("ddagrab=output_idx={}:framerate={}", self.monitor_index, self.framerate);
        if let Some(size) = &self.video_size {
            filter_opts.push_str(&format!(":video_size={}", size));
        }
        args.push("-i".to_string());
        args.push(filter_opts);

        // EXTRA HW FRAMES (Prevents GPU Stalls)
        args.push("-extra_hw_frames".to_string());
        args.push(FFMPEG_EXTRA_HW_FRAMES.to_string());

        // Input 1: Microphone (Pipe)
        if let Some(mic_rate) = &self.mic_sample_rate {
            let mic_ch = self.mic_channels.unwrap_or(1).to_string();
            args.extend(vec![
                "-f".to_string(), "f32le".to_string(),
                "-ar".to_string(), mic_rate.to_string(),
                "-ac".to_string(), mic_ch,
                "-thread_queue_size".to_string(), FFMPEG_THREAD_QUEUE_SIZE.to_string(),
                "-i".to_string(), "pipe:0".to_string(),
            ]);
        }

        // Input 2: System Audio (Named Pipe)
        if self.system_audio {
            let sys_ch = self.system_channels.unwrap_or(2).to_string();
            args.extend(vec![
                "-f".to_string(), "f32le".to_string(),
                "-ar".to_string(), self.system_sample_rate.to_string(),
                "-ac".to_string(), sys_ch,
                "-thread_queue_size".to_string(), FFMPEG_THREAD_QUEUE_SIZE.to_string(),
                "-i".to_string(), SYSTEM_AUDIO_PIPE_NAME.to_string(),
            ]);
        }

        // --- FILTER CHAIN ---
        let mut video_filters = String::new();
        
        // Resolution Logic
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
            video_filters.push_str("scale_d3d11=format=nv12");
        }

        let has_mic = self.audio_source.is_some();
        let has_sys = self.system_audio;

        if has_mic || has_sys {
            let video_chain = if !video_filters.is_empty() {
                format!("[0:v]setpts=PTS-STARTPTS,{}[vout]", video_filters)
            } else {
                "[0:v]setpts=PTS-STARTPTS[vout]".to_string()
            };

            // Audio Mixing Logic
            // Inputs:
            // 0: Video
            // 1: Mic (if exists)
            // 2: System (if exists, or 1 if no mic)
            
            let audio_chain = if has_mic && has_sys {
                // Mix both
                // CRITICAL: Force Mic to Stereo (ochl=stereo) so amix doesn't collapse System Audio to Mono.
                // Decouple inputs: duration=first (Mic is master), dropout_transition=0 (instant recovery)
                // RESTORED: async=10000 and first_pts=0 to fix drift and align start time.
                "[1:a]aresample=48000:ochl=stereo,aresample=async=10000:first_pts=0[a1];[2:a]aresample=48000:ochl=stereo,aresample=async=10000:first_pts=0[a2];[a1][a2]amix=inputs=2:duration=first:dropout_transition=0[mixed];[mixed]asetpts=PTS-STARTPTS[aout]".to_string()
            } else if has_mic {
                // Just Mic - Force Stereo for consistency
                "[1:a]aresample=48000:ochl=stereo,aresample=async=10000:first_pts=0,asetpts=PTS-STARTPTS[aout]".to_string()
            } else {
                // Just System
                "[1:a]aresample=48000,aresample=async=10000:first_pts=0,asetpts=PTS-STARTPTS[aout]".to_string()
            };

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
                "-b:v".to_string(), self.bitrate.clone(),
                "-maxrate".to_string(), format!("{}M", self.bitrate.replace("M", "").parse::<u32>().unwrap_or(8) * 3 / 2),
                "-bufsize".to_string(), format!("{}M", self.bitrate.replace("M", "").parse::<u32>().unwrap_or(8) * 2),
                "-preset".to_string(), self.preset.clone().unwrap_or("p1".to_string()),
                "-profile:v".to_string(), self.profile.clone().unwrap_or("high".to_string()),
            ]);
            // Only add tune if explicitly set (removed default 'ull' for quality)
            if let Some(tune) = &self.tune {
                 args.extend(vec!["-tune".to_string(), tune.clone()]);
            }
        } else if self.video_codec.contains("amf") {
            args.extend(vec![
                "-rc".to_string(), "cbr".to_string(),
                "-b:v".to_string(), self.bitrate.clone(),
                "-usage".to_string(), "transcoding".to_string(),
                "-quality".to_string(), self.preset.clone().unwrap_or("speed".to_string()),
                "-profile:v".to_string(), self.profile.clone().unwrap_or("high".to_string()),
            ]);
        } else if self.video_codec.contains("qsv") {
            args.extend(vec![
                "-rc".to_string(), "vbr".to_string(),
                "-b:v".to_string(), self.bitrate.clone(),
                "-preset".to_string(), self.preset.clone().unwrap_or("veryfast".to_string()),
                "-profile:v".to_string(), self.profile.clone().unwrap_or("high".to_string()),
            ]);
        } else {
            args.extend(vec![
                "-b:v".to_string(), self.bitrate.clone(),
                "-preset".to_string(), self.preset.clone().unwrap_or("ultrafast".to_string()),
                "-tune".to_string(), self.tune.clone().unwrap_or("zerolatency".to_string()),
            ]);
        }

        // --- FRAMERATE ---
        args.extend(vec![
            "-fps_mode".to_string(), "vfr".to_string(),
        ]);

        // --- GOP / KEYFRAMES ---
        let gop = self.framerate * 2;
        args.extend(vec![
            "-g".to_string(), gop.to_string(),
            "-bf".to_string(), "2".to_string(), // Enable B-frames for quality
        ]);

        // --- AUDIO ENCODING ---
        if has_mic || has_sys {
            args.extend(vec![
                "-c:a".to_string(), self.audio_codec.clone().unwrap_or_else(|| "aac".to_string()),
                "-b:a".to_string(), self.audio_bitrate.clone().unwrap_or("192k".to_string()),
            ]);
        }

        // --- OUTPUT ---
        args.extend(vec![
            "-movflags".to_string(), "+frag_keyframe+empty_moov+default_base_moof".to_string(),
            "-max_muxing_queue_size".to_string(), "9999".to_string(),
            "-stats".to_string(),
        ]);

        if let (Some(time), Some(wrap), Some(list)) = (self.segment_time, self.segment_wrap, &self.segment_list) {
            // Segment Muxer Output
            args.extend(vec![
                "-f".to_string(), "segment".to_string(),
                "-segment_format".to_string(), "mpegts".to_string(),
                "-segment_time".to_string(), time.to_string(),
                "-segment_wrap".to_string(), wrap.to_string(),
                "-segment_list".to_string(), list.clone(),
                "-segment_list_type".to_string(), "m3u8".to_string(),
                "-reset_timestamps".to_string(), "1".to_string(), // Crucial for stitching
                self.output_path.clone(), // This should be a pattern like "clip_%03d.ts"
            ]);
        } else {
            // Standard MP4 Output
            args.extend(vec![
                "-f".to_string(), "mp4".to_string(),
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
            .with_video_codec("h264_nvenc".to_string())
            .with_tune(Some("ull".to_string())); // Explicitly add tune for test
        let args = builder.build();
        
        assert!(args.contains(&"h264_nvenc".to_string()));
        assert!(args.contains(&"-tune".to_string()));
        assert!(args.contains(&"ull".to_string())); 
    }

    #[test]
    fn test_builder_audio() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_audio(Some("Mic".to_string()), false, 48000, Some(48000), Some(2), Some(2), Some("aac".to_string()), None);
        let args = builder.build();
        
        assert!(args.contains(&"pipe:0".to_string()));
        assert!(args.contains(&"aac".to_string()));
        // Check for filter complex
        let has_filter = args.iter().any(|a| a.contains("aresample"));
        assert!(has_filter);
    }
}
