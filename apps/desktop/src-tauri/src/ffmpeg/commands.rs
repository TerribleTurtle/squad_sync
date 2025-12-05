//! FFmpeg Command Builder
//! 
//! This module provides the [FfmpegCommandBuilder] struct for constructing
//! complex FFmpeg CLI arguments. It is primarily used by [crate::ffmpeg::process].

use crate::constants::{
    SYSTEM_AUDIO_PIPE_NAME, MIC_AUDIO_PIPE_NAME,
    FFMPEG_THREAD_QUEUE_SIZE, FFMPEG_AUDIO_THREAD_QUEUE_SIZE, FFMPEG_EXTRA_HW_FRAMES, FFMPEG_MAX_MUXING_QUEUE_SIZE,
    DEFAULT_VIDEO_CODEC, DEFAULT_VIDEO_BITRATE, DEFAULT_VIDEO_FRAMERATE,
    DEFAULT_AUDIO_SAMPLE_RATE, DEFAULT_AUDIO_CHANNELS,
    AUDIO_BUFFER_SIZE_MS, RTBUFSIZE, PRESET_P4,
    TUNE_ZEROLATENCY, PROFILE_HIGH,
    SEGMENT_LIST_SIZE, SEGMENT_LIST_TYPE, SEGMENT_FORMAT_MKV,
    OUTPUT_FORMAT_SEGMENT, OUTPUT_FORMAT_MP4, OUTPUT_FORMAT_LAVFI, OUTPUT_FORMAT_DSHOW, OUTPUT_FORMAT_F32LE,
    BITRATE_MAX_MULTIPLIER, BITRATE_MAX_DIVISOR, BITRATE_BUF_MULTIPLIER, GOP_MULTIPLIER,
    DEFAULT_MIC_CHANNELS
};
use crate::ffmpeg::encoder::HardwareScalingMode;

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

    audio_backend: String,
    
    // Segment Config
    segment_time: Option<u32>,
    segment_wrap: Option<u32>,
    segment_list: Option<String>,
    mode: CommandMode,
    scaling_mode: HardwareScalingMode,
}



#[derive(Debug, Clone, PartialEq)]
pub enum CommandMode {
    Combined,
    VideoOnly,
    AudioOnly,
}


impl FfmpegCommandBuilder {
    pub fn new(output_path: String) -> Self {
        Self {
            framerate: DEFAULT_VIDEO_FRAMERATE,
            video_codec: DEFAULT_VIDEO_CODEC.to_string(),
            bitrate: DEFAULT_VIDEO_BITRATE.to_string(),
            preset: None,
            tune: None,
            profile: None,
            output_path,
            resolution: None,
            video_size: None,
            monitor_index: 0,
            audio_source: None,
            system_audio: false,
            system_sample_rate: DEFAULT_AUDIO_SAMPLE_RATE,
            mic_sample_rate: None,
            mic_channels: None,
            system_channels: None,
            audio_codec: None,
            audio_bitrate: None,
            audio_sample_rate: DEFAULT_AUDIO_SAMPLE_RATE,
            audio_channels: DEFAULT_AUDIO_CHANNELS,

            audio_backend: "cpal".to_string(),
            segment_time: None,
            segment_wrap: None,
            segment_list: None,
            mode: CommandMode::Combined,
            scaling_mode: HardwareScalingMode::None,
        }
    }

    pub fn with_scaling_mode(mut self, mode: HardwareScalingMode) -> Self {
        self.scaling_mode = mode;
        self
    }

    pub fn with_mode(mut self, mode: CommandMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_output_path(mut self, path: String) -> Self {
        self.output_path = path;
        self
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

    pub fn with_audio_source(mut self, source: Option<String>) -> Self {
        self.audio_source = source;
        self
    }

    pub fn with_system_audio(mut self, enabled: bool) -> Self {
        self.system_audio = enabled;
        self
    }

    pub fn with_audio_input_config(mut self, system_rate: u32, mic_rate: Option<u32>, mic_channels: Option<u16>, system_channels: Option<u16>) -> Self {
        self.system_sample_rate = system_rate;
        self.mic_sample_rate = mic_rate;
        self.mic_channels = mic_channels;
        self.system_channels = system_channels;
        self
    }

    pub fn with_audio_output_config(mut self, codec: Option<String>, bitrate: Option<String>, sample_rate: u32, channels: u16) -> Self {
        self.audio_codec = codec;
        self.audio_bitrate = bitrate;
        self.audio_sample_rate = sample_rate;
        self.audio_channels = channels;
        self
    }


    pub fn with_audio_backend(mut self, backend: String) -> Self {
        self.audio_backend = backend;
        self
    }

    pub fn with_segment_config(mut self, time: u32, wrap: u32, list: String) -> Self {
        self.segment_time = Some(time);
        self.segment_wrap = Some(wrap);
        self.segment_list = Some(list);
        self
    }

    pub fn get_segment_time(&self) -> Option<u32> {
        self.segment_time
    }

    pub fn get_segment_wrap(&self) -> Option<u32> {
        self.segment_wrap
    }

    pub fn build(&self) -> Vec<String> {
        let mut args = Vec::new();
        
        match self.mode {
            CommandMode::Combined => {
                args.extend(self.build_inputs());
                args.extend(self.build_filter_chain());
                args.extend(self.build_encoding_options());
                args.extend(self.build_output());
            }
            CommandMode::VideoOnly => {
                args.extend(self.build_video_inputs());
                args.extend(self.build_video_filters());
                args.extend(self.build_video_encoding());
                args.extend(self.build_output());
            }
            CommandMode::AudioOnly => {
                args.extend(self.build_audio_inputs());
                // No complex filters for audio-only usually, but we might need aresample
                args.extend(self.build_audio_filters());
                args.extend(self.build_audio_encoding());
                args.extend(self.build_output());
            }
        }

        args
    }

    // --- VIDEO ONLY HELPERS ---
    fn build_video_inputs(&self) -> Vec<String> {
        let mut args = vec![
            "-f".to_string(),
            OUTPUT_FORMAT_LAVFI.to_string(),
            "-thread_queue_size".to_string(),
            FFMPEG_THREAD_QUEUE_SIZE.to_string(),
        ];
        
        let mut filter_opts = format!("ddagrab=output_idx={}", self.monitor_index);
        if let Some(size) = &self.video_size {
            filter_opts.push_str(&format!(":video_size={}", size));
        }
        
        // EXTRA HW FRAMES
        args.push("-extra_hw_frames".to_string());
        args.push(FFMPEG_EXTRA_HW_FRAMES.to_string());

        // Wallclock Timestamps (Sync)
        args.push("-use_wallclock_as_timestamps".to_string());
        args.push("1".to_string());

        args.push("-i".to_string());
        args.push(filter_opts);

        args
    }

    fn build_video_filters(&self) -> Vec<String> {
        let mut args = Vec::new();
        let mut video_filters = String::new();
        let mut is_hardware_frame = true; // ddagrab starts in D3D11

        // Resolution Logic
        let use_native_res = match &self.resolution {
            Some(r) => r.to_lowercase() == "native",
            None => true,
        };

        if !use_native_res {
            if let Some(res) = &self.resolution {
                let parts: Vec<&str> = res.split('x').collect();
                if parts.len() == 2 {
                    match self.scaling_mode {
                        HardwareScalingMode::D3D11 => {
                            video_filters.push_str(&format!("scale_d3d11=width={}:height={}:format=nv12", parts[0], parts[1]));
                            // Still D3D11
                        },
                        HardwareScalingMode::CUDA => {
                            // D3D11 (Input) -> Map to CUDA -> Scale CUDA -> NVENC (Native CUDA)
                            video_filters.push_str("hwmap=derive_device=cuda,");
                            video_filters.push_str(&format!("scale_cuda=w={}:h={}:format=nv12", parts[0], parts[1]));
                            // Now CUDA
                        },
                        HardwareScalingMode::None => {
                            // Fallback to software scale
                            // hwdownload -> format (force bgra/rgba to prevent nv12 negotiation error) -> scale
                            video_filters.push_str("hwdownload,format=bgra,");
                            video_filters.push_str(&format!("scale={}:{}", parts[0], parts[1]));
                            is_hardware_frame = false;
                        }
                    }
                }
            }
        }

        if !video_filters.is_empty() {
            // Ensure comma separator if we have previous filters
            if !video_filters.ends_with(',') {
                video_filters.push(',');
            }
        }

        // --- FORMAT BRIDGING ---
        // ddagrab outputs D3D11 surfaces. We need to bridge them to the encoder's expected format.
        
        if self.video_codec.contains("qsv") {
            if is_hardware_frame {
                // QSV needs a QSV surface derived from D3D11
                video_filters.push_str("hwmap=derive_device=qsv,format=qsv");
            } else {
                // Already software (e.g. after software scale). 
                // QSV encoder can handle system memory (nv12), so no extra bridge needed usually.
                // But ensuring nv12 is good.
                video_filters.push_str("format=nv12");
            }
        } else if self.video_codec.contains("libx264") {
            if is_hardware_frame {
                // Software encoding needs system memory
                // hwdownload -> format (force bgra/rgba) -> format=nv12 (for encoder)
                video_filters.push_str("hwdownload,format=bgra,format=nv12");
                is_hardware_frame = false;
            } else {
                // Already software
                video_filters.push_str("format=nv12");
            }
        } 
        // NVENC and AMF (usually) support D3D11 input directly, so no bridging needed if is_hardware_frame.
        // If !is_hardware_frame (software scaled), NVENC can also handle system memory.
        if !is_hardware_frame && (self.video_codec.contains("nvenc") || self.video_codec.contains("amf")) {
             // Ensure we are in a friendly format (nv12) if we dropped to software
             video_filters.push_str("format=nv12");
        }

        if !video_filters.is_empty() {
            // Clean up trailing comma if any
             if video_filters.ends_with(',') {
                video_filters.pop();
            }
            
            args.push("-vf".to_string());
            args.push(video_filters);
        }
        
        args
    }

    fn build_video_encoding(&self) -> Vec<String> {
        let mut args = self.build_encoding_options_video_part();
        // Disable Audio
        args.push("-an".to_string());
        args
    }

    // --- AUDIO ONLY HELPERS ---
    fn build_audio_inputs(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Input 0: Microphone
        if let Some(mic_source) = &self.audio_source {
            if self.audio_backend == "dshow" {
                 args.extend(vec![
                     "-f".to_string(), OUTPUT_FORMAT_DSHOW.to_string(),
                    "-audio_buffer_size".to_string(), AUDIO_BUFFER_SIZE_MS.to_string(),
                    "-thread_queue_size".to_string(), FFMPEG_AUDIO_THREAD_QUEUE_SIZE.to_string(),
                    "-rtbufsize".to_string(), RTBUFSIZE.to_string(),
                    "-use_wallclock_as_timestamps".to_string(), "1".to_string(),
                    "-i".to_string(), format!("audio={}", mic_source),
                 ]);
            } else if let Some(mic_rate) = &self.mic_sample_rate {
                let mic_ch = self.mic_channels.unwrap_or(DEFAULT_MIC_CHANNELS).to_string();
                args.extend(vec![
                    "-f".to_string(), OUTPUT_FORMAT_F32LE.to_string(),
                    "-thread_queue_size".to_string(), FFMPEG_AUDIO_THREAD_QUEUE_SIZE.to_string(),
                    "-ar".to_string(), mic_rate.to_string(),
                    "-ac".to_string(), mic_ch,
                    "-use_wallclock_as_timestamps".to_string(), "1".to_string(),
                    "-i".to_string(), MIC_AUDIO_PIPE_NAME.to_string(),
                ]);
            }
        }

        // Input 1: System Audio
        if self.system_audio {
            let sys_ch = self.system_channels.unwrap_or(DEFAULT_AUDIO_CHANNELS).to_string();
            args.extend(vec![
                "-f".to_string(), OUTPUT_FORMAT_F32LE.to_string(),
                "-thread_queue_size".to_string(), FFMPEG_AUDIO_THREAD_QUEUE_SIZE.to_string(),
                "-ar".to_string(), self.system_sample_rate.to_string(),
                "-ac".to_string(), sys_ch,
                "-use_wallclock_as_timestamps".to_string(), "1".to_string(),
                "-i".to_string(), SYSTEM_AUDIO_PIPE_NAME.to_string(),
            ]);
        }

        args
    }

    fn build_audio_filters(&self) -> Vec<String> {
        let mut args = Vec::new();
        
        let has_mic = self.audio_source.is_some();
        let has_sys = self.system_audio;

        if has_mic || has_sys {
            // Audio Mixing Logic
            // Note: Input indices depend on what was added in build_audio_inputs
            // If both: 0 is Mic, 1 is Sys
            // If only Mic: 0 is Mic
            // If only Sys: 0 is Sys
            
            let audio_chain = if has_mic && has_sys {
                // Mix both (Mic is 0, Sys is 1)
                format!("[0:a]aresample={}:resampler=soxr,aformat=channel_layouts=stereo,aresample=async=1:first_pts=0[a1];[1:a]aresample={}:resampler=soxr,aformat=channel_layouts=stereo,aresample=async=1:first_pts=0[a2];[a1][a2]amix=inputs=2:duration=longest:dropout_transition=0[aout]", 
                    self.audio_sample_rate, self.audio_sample_rate)
            } else if has_mic {
                // Just Mic (Input 0)
                format!("[0:a]aresample={}:resampler=soxr,aformat=channel_layouts=stereo,aresample=async=1:first_pts=0[aout]", 
                    self.audio_sample_rate)
            } else {
                // Just System (Input 0)
                format!("[0:a]aresample={},aresample=async=1:first_pts=0[aout]", self.audio_sample_rate)
            };

            args.extend(vec![
                "-filter_complex".to_string(), audio_chain,
                "-map".to_string(), "[aout]".to_string(),
            ]);
        }

        args
    }

    fn build_audio_encoding(&self) -> Vec<String> {
        let mut args = Vec::new();
        // Disable Video
        args.push("-vn".to_string());
        
        let has_mic = self.audio_source.is_some();
        let has_sys = self.system_audio;

        if has_mic || has_sys {
            args.extend(vec![
                "-c:a".to_string(), "pcm_s16le".to_string(),
            ]);
        }
        args
    }

    // --- SHARED / LEGACY ---
    fn build_inputs(&self) -> Vec<String> {
        // Legacy Combined Logic - Preserved for backward compatibility
        // but refactored to use helper methods to avoid duplication.
        
        let mut args = Vec::new();
        
        // Video Input
        args.extend(self.build_video_inputs());

        // Audio Inputs
        args.extend(self.build_audio_inputs());

        args
    }


    fn build_filter_chain(&self) -> Vec<String> {
        let mut args = Vec::new();
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
                    match self.scaling_mode {
                        HardwareScalingMode::D3D11 => {
                             video_filters.push_str(&format!("scale_d3d11=width={}:height={}:format=nv12", parts[0], parts[1]));
                        },
                        HardwareScalingMode::CUDA => {
                             video_filters.push_str("hwmap=derive_device=cuda,");
                             video_filters.push_str(&format!("scale_cuda=w={}:h={}:format=nv12", parts[0], parts[1]));
                        },
                        HardwareScalingMode::None => {
                             video_filters.push_str("hwdownload,");
                             video_filters.push_str(&format!("scale={}:{}", parts[0], parts[1]));
                        }
                    }
                }
            }
        }

        // REMOVED: fps filter
        // video_filters.push_str(&format!("fps={}", self.framerate));

        if !video_filters.is_empty()
             && !video_filters.ends_with(',') {
                video_filters.push(',');
            }

        // --- FORMAT BRIDGING ---
        if self.video_codec.contains("qsv") {
            video_filters.push_str("hwmap=derive_device=qsv,format=qsv");
        } else if self.video_codec.contains("libx264") {
            video_filters.push_str("hwdownload,format=nv12");
        }

        if !video_filters.is_empty()
             && video_filters.ends_with(',') {
                video_filters.pop();
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
            let audio_chain = if has_mic && has_sys {
                // Mix both
                format!("[1:a]aresample={}:resampler=soxr,aformat=channel_layouts=stereo,aresample=async=1:first_pts=0,asetpts=PTS-STARTPTS[a1];[2:a]aresample={}:resampler=soxr,aformat=channel_layouts=stereo,aresample=async=1:first_pts=0,asetpts=PTS-STARTPTS[a2];[a1][a2]amix=inputs=2:duration=longest:dropout_transition=0[mixed];[mixed]asetpts=PTS-STARTPTS[aout]", 
                    self.audio_sample_rate, self.audio_sample_rate)
            } else if has_mic {
                // Just Mic
                format!("[1:a]aresample={}:resampler=soxr,aformat=channel_layouts=stereo,aresample=async=1:first_pts=0,asetpts=PTS-STARTPTS[aout]", 
                    self.audio_sample_rate)
            } else {
                // Just System
                format!("[1:a]aresample={},aresample=async=1:first_pts=0,asetpts=PTS-STARTPTS[aout]", self.audio_sample_rate)
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

        args
    }

    fn build_encoding_options(&self) -> Vec<String> {
        let mut args = self.build_encoding_options_video_part();
        args.extend(self.build_encoding_options_audio_part());
        args
    }

    fn build_encoding_options_video_part(&self) -> Vec<String> {
        let mut args = Vec::new();

        // --- VIDEO ENCODING ---
        args.extend(vec!["-c:v".to_string(), self.video_codec.clone()]);

        // Restore pix_fmt d3d11 for hardware encoders (ddagrab path)
        // Previous behavior was to set d3d11 for ddagrab. Since ddagrab is now default/hardcoded,
        // we set it for all hardware encoders. Software (x264) needs yuv420p.
        // WARNING: DO NOT TOUCH THIS WITHOUT EXPLICIT PERMISSION.
        // Changing this will break scale_d3d11 and cause A/V desync.
        if !self.video_codec.contains("libx264") {
             args.extend(vec!["-pix_fmt".to_string(), "d3d11".to_string()]);
        } else {
             args.extend(vec!["-pix_fmt".to_string(), "yuv420p".to_string()]);
        }
        
        // Sanitize preset based on codec
        let raw_preset = self.preset.clone().unwrap_or(PRESET_P4.to_string());
        let preset = Self::sanitize_preset(&self.video_codec, &raw_preset);

        if self.video_codec.contains("nvenc") {
            // Use shared utility for bitrate parsing
            let kbps = crate::ffmpeg::utils::parse_bitrate(&self.bitrate) / 1000;
            args.extend(vec![
                "-rc".to_string(), "vbr".to_string(),
                "-b:v".to_string(), self.bitrate.clone(),
                "-maxrate".to_string(), format!("{}k", kbps * BITRATE_MAX_MULTIPLIER / BITRATE_MAX_DIVISOR),
                "-bufsize".to_string(), format!("{}k", kbps * BITRATE_BUF_MULTIPLIER),
                "-preset".to_string(), preset,
                "-profile:v".to_string(), self.profile.clone().unwrap_or(PROFILE_HIGH.to_string()),
            ]);
            if let Some(tune) = &self.tune {
                 args.extend(vec!["-tune".to_string(), tune.clone()]);
            }
        } else if self.video_codec.contains("amf") {
            args.extend(vec![
                "-rc".to_string(), "cbr".to_string(),
                "-b:v".to_string(), self.bitrate.clone(),
                "-usage".to_string(), "transcoding".to_string(),
                "-quality".to_string(), preset, // AMF uses -quality, not -preset
                "-profile:v".to_string(), self.profile.clone().unwrap_or(PROFILE_HIGH.to_string()),
            ]);
        } else if self.video_codec.contains("qsv") {
            args.extend(vec![
                "-rc".to_string(), "vbr".to_string(),
                "-b:v".to_string(), self.bitrate.clone(),
                "-preset".to_string(), preset,
                "-profile:v".to_string(), self.profile.clone().unwrap_or(PROFILE_HIGH.to_string()),
            ]);
        } else {
            args.extend(vec![
                "-b:v".to_string(), self.bitrate.clone(),
                "-preset".to_string(), preset,
                "-tune".to_string(), self.tune.clone().unwrap_or(TUNE_ZEROLATENCY.to_string()),
            ]);
        }

        // --- FRAMERATE ---
        // Use -r for output framerate (replaces fps filter)
        args.extend(vec![
            "-r".to_string(), self.framerate.to_string(),
            "-fps_mode".to_string(), "cfr".to_string(), // Enforce constant frame rate
        ]);

        // --- GOP / KEYFRAMES ---
        let gop = self.framerate * GOP_MULTIPLIER;
        args.extend(vec![
            "-g".to_string(), gop.to_string(),
            "-bf".to_string(), "0".to_string(),
        ]);

        // Force Keyframes at Segment Boundaries
        if let Some(segment_time) = self.segment_time {
            args.extend(vec![
                "-force_key_frames".to_string(),
                format!("expr:gte(t,n_forced*{})", segment_time),
            ]);
        }

        args
    }

    fn build_encoding_options_audio_part(&self) -> Vec<String> {
        let mut args = Vec::new();
        // --- AUDIO ENCODING ---
        let has_mic = self.audio_source.is_some();
        let has_sys = self.system_audio;

        if has_mic || has_sys {
            args.extend(vec![
                "-c:a".to_string(), "pcm_s16le".to_string(),
            ]);
        }
        args
    }


    fn build_output(&self) -> Vec<String> {
        let mut args = Vec::new();

        args.extend(vec![
            "-max_muxing_queue_size".to_string(), FFMPEG_MAX_MUXING_QUEUE_SIZE.to_string(),
            "-stats".to_string(),
        ]);

        if let (Some(time), Some(_wrap), Some(list)) = (self.segment_time, self.segment_wrap, &self.segment_list) {
            // Segment Muxer Output
            args.extend(vec![
                "-flush_packets".to_string(), "1".to_string(), // Force immediate write
                "-f".to_string(), OUTPUT_FORMAT_SEGMENT.to_string(),
                "-segment_time".to_string(), time.to_string(),
                "-segment_list_size".to_string(), SEGMENT_LIST_SIZE.to_string(),
                "-segment_list".to_string(), list.clone(),
                "-segment_list_type".to_string(), SEGMENT_LIST_TYPE.to_string(),
                "-segment_list_flags".to_string(), "+live".to_string(), // Update playlist immediately
                "-segment_format".to_string(), SEGMENT_FORMAT_MKV.to_string(),
                "-strftime".to_string(), "1".to_string(), // Enable strftime expansion
                "-reset_timestamps".to_string(), "1".to_string(),
                self.output_path.clone(),
            ]);
        } else {
            // Standard MP4 Output
            args.extend(vec![
                "-f".to_string(), OUTPUT_FORMAT_MP4.to_string(),
                self.output_path.clone(),
            ]);
        }
        
        args
    }

    fn sanitize_preset(codec: &str, preset: &str) -> String {
        let p = preset.to_lowercase();
        
        if codec.contains("nvenc") {
            // NVENC expects p1-p7
            match p.as_str() {
                "speed" | "veryfast" | "faster" | "fast" => "p2".to_string(),
                "balanced" | "medium" => "p4".to_string(),
                "quality" | "slow" | "slower" | "veryslow" => "p6".to_string(),
                val if val.starts_with('p') && val.len() == 2 => val.to_string(), // p1-p7
                _ => "p4".to_string(),
            }
        } else if codec.contains("amf") {
            // AMF expects speed, balanced, quality
            match p.as_str() {
                "p1" | "p2" | "veryfast" | "faster" | "fast" | "speed" => "speed".to_string(),
                "p3" | "p4" | "medium" | "balanced" => "balanced".to_string(),
                "p5" | "p6" | "p7" | "slow" | "slower" | "veryslow" | "quality" => "quality".to_string(),
                _ => "balanced".to_string(),
            }
        } else if codec.contains("qsv") {
            // QSV expects veryfast, medium, veryslow
            match p.as_str() {
                "p1" | "p2" | "speed" | "veryfast" | "faster" | "fast" => "veryfast".to_string(),
                "p3" | "p4" | "balanced" | "medium" => "medium".to_string(),
                "p5" | "p6" | "p7" | "quality" | "slow" | "slower" | "veryslow" => "veryslow".to_string(),
                _ => "veryfast".to_string(),
            }
        } else {
            // Software (x264) - Standard presets
            match p.as_str() {
                "p1" | "p2" | "speed" => "ultrafast".to_string(),
                "p3" | "p4" | "balanced" => "veryfast".to_string(),
                "p5" | "p6" | "p7" | "quality" => "medium".to_string(),
                _ => p, // Pass through standard presets (ultrafast, veryfast, etc.)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string());
        let args = builder.build();
        
        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(&"lavfi".to_string()));
        assert!(args.contains(&"output.mp4".to_string()));
        assert!(args.contains(&DEFAULT_VIDEO_CODEC.to_string()));
    }

    #[test]
    fn test_builder_nvenc() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_video_codec("h264_nvenc".to_string())
            .with_tune(Some("ull".to_string()));
        let args = builder.build();
        
        assert!(args.contains(&"h264_nvenc".to_string()));
        assert!(args.contains(&"-tune".to_string()));
        assert!(args.contains(&"ull".to_string())); 
    }

    #[test]
    fn test_builder_audio() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_audio_source(Some("Mic".to_string()))
            .with_audio_input_config(48000, Some(48000), Some(2), Some(2))
            .with_audio_output_config(Some("aac".to_string()), None, 48000, 2);
        let args = builder.build();
        
        assert!(args.contains(&MIC_AUDIO_PIPE_NAME.to_string()));
        // Should now be pcm_s16le regardless of input config for recording
        assert!(args.contains(&"pcm_s16le".to_string()));
        let has_filter = args.iter().any(|a| a.contains("aresample"));
        assert!(has_filter);
    }

    #[test]
    fn test_builder_system_audio_only() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_system_audio(true)
            .with_audio_input_config(48000, None, None, Some(2))
            .with_audio_output_config(Some("aac".to_string()), None, 48000, 2);
        let args = builder.build();

        assert!(args.contains(&SYSTEM_AUDIO_PIPE_NAME.to_string()));
        assert!(!args.contains(&MIC_AUDIO_PIPE_NAME.to_string()));
        assert!(args.contains(&"pcm_s16le".to_string()));
        
        // Check for specific filter chain parts
        let filter_complex = args.iter().position(|r| r == "-filter_complex").unwrap();
        let filter_chain = &args[filter_complex + 1];
        assert!(filter_chain.contains("[1:a]aresample="));
        assert!(!filter_chain.contains("amix")); // No mixing if only one source
    }

    #[test]
    fn test_builder_mixed_audio() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_audio_source(Some("Mic".to_string()))
            .with_system_audio(true)
            .with_audio_input_config(48000, Some(48000), Some(2), Some(2))
            .with_audio_output_config(Some("aac".to_string()), None, 48000, 2);
        let args = builder.build();

        assert!(args.contains(&MIC_AUDIO_PIPE_NAME.to_string()));
        assert!(args.contains(&SYSTEM_AUDIO_PIPE_NAME.to_string()));
        assert!(args.contains(&"pcm_s16le".to_string()));
        
        let filter_complex = args.iter().position(|r| r == "-filter_complex").unwrap();
        let filter_chain = &args[filter_complex + 1];
        assert!(filter_chain.contains("amix=inputs=2"));
    }

    #[test]
    fn test_bitrate_calculation_nvenc() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_video_codec("h264_nvenc".to_string())
            .with_bitrate("10M".to_string());
        let args = builder.build();

        // 10M = 10,000k
        // maxrate = 10000 * 1.5 = 15000k
        // bufsize = 10000 * 2 = 20000k

        let maxrate_idx = args.iter().position(|r| r == "-maxrate").unwrap();
        assert_eq!(args[maxrate_idx + 1], "15000k");

        let bufsize_idx = args.iter().position(|r| r == "-bufsize").unwrap();
        assert_eq!(args[bufsize_idx + 1], "20000k");
    }

    #[test]
    fn test_builder_audio_only_codec() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_mode(CommandMode::AudioOnly)
            .with_audio_source(Some("Mic".to_string()))
            .with_audio_input_config(48000, Some(48000), Some(2), Some(2));
        let args = builder.build();

        assert!(args.contains(&"pcm_s16le".to_string()));
        assert!(!args.contains(&"aac".to_string()));
        assert!(!args.contains(&"-b:a".to_string()));
    }

    #[test]
    fn test_bitrate_calculation_kbps() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_video_codec("h264_nvenc".to_string())
            .with_bitrate("5000k".to_string());
        let args = builder.build();

        // 5000k
        // maxrate = 5000 * 1.5 = 7500k
        // bufsize = 5000 * 2 = 10000k

        let maxrate_idx = args.iter().position(|r| r == "-maxrate").unwrap();
        assert_eq!(args[maxrate_idx + 1], "7500k");

        let bufsize_idx = args.iter().position(|r| r == "-bufsize").unwrap();
        assert_eq!(args[bufsize_idx + 1], "10000k");
    }


    #[test]
    fn test_builder_resolution_edge_cases() {
        // Case 1: "Native" mixed case
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_video_codec("h264_nvenc".to_string()) // Explicitly set HW codec to avoid format bridging filters
            .with_resolution(Some("Native".to_string()));
        let args = builder.build();
        
        // Native resolution should NOT add any scaling filters
        let has_vf = args.contains(&"-vf".to_string());
        let has_fc = args.contains(&"-filter_complex".to_string());
        assert!(!has_vf && !has_fc, "Native resolution should not add filters");

        // Case 2: Invalid resolution string
        let builder2 = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_video_codec("h264_nvenc".to_string()) // Explicitly set HW codec
            .with_resolution(Some("invalid".to_string()));
        let args2 = builder2.build();
        
        // Invalid resolution should fallback to Native (no filters), not panic
        let has_vf2 = args2.contains(&"-vf".to_string());
        let has_fc2 = args2.contains(&"-filter_complex".to_string());
        assert!(!has_vf2 && !has_fc2, "Invalid resolution should fallback to no filters");
    }

    #[test]
    fn test_builder_segment_config() {
        let builder = FfmpegCommandBuilder::new("output_%03d.mkv".to_string())
            .with_segment_config(60, 5, "list.m3u8".to_string());
        let args = builder.build();
        
        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(&"segment".to_string()));
        assert!(args.contains(&"matroska".to_string()));
        assert!(args.contains(&"-segment_time".to_string()));
        assert!(args.contains(&"60".to_string()));
    }

    #[test]
    fn test_builder_mkv_migration() {
        let builder = FfmpegCommandBuilder::new("output.mkv".to_string())
            .with_audio_source(Some("Mic".to_string()))
            .with_audio_output_config(Some("pcm_s16le".to_string()), None, 48000, 2);
        let args = builder.build();

        // Check for Wallclock
        assert!(args.contains(&"-use_wallclock_as_timestamps".to_string()));
        
        // Check for PCM
        assert!(args.contains(&"pcm_s16le".to_string()));
    }


    #[test]
    fn test_builder_video_only() {
        let builder = FfmpegCommandBuilder::new("video_%03d.ts".to_string())
            .with_mode(CommandMode::VideoOnly)
            .with_video_codec("h264_nvenc".to_string())
            .with_audio_source(Some("Mic".to_string())); // Should be ignored/disabled
        
        let args = builder.build();
        
        // Check for Video stuff
        assert!(args.contains(&"h264_nvenc".to_string()));
        assert!(args.contains(&"-an".to_string())); // Audio Disabled
        
        // Check for NO Audio stuff
        assert!(!args.contains(&"-c:a".to_string()));
        assert!(!args.contains(&MIC_AUDIO_PIPE_NAME.to_string()));
        assert!(!args.contains(&"aac".to_string()));
    }

    #[test]
    fn test_builder_audio_only() {
        let builder = FfmpegCommandBuilder::new("audio_%03d.ts".to_string())
            .with_mode(CommandMode::AudioOnly)
            .with_audio_source(Some("Mic".to_string()))
            .with_audio_input_config(48000, Some(48000), Some(1), Some(2))
            .with_video_codec("h264_nvenc".to_string()); // Should be ignored
            
        let args = builder.build();
        
        // Check for Audio stuff
        assert!(args.contains(&"-vn".to_string())); // Video Disabled
        assert!(args.contains(&MIC_AUDIO_PIPE_NAME.to_string()));
        
        // Check for NO Video stuff
        assert!(!args.contains(&"h264_nvenc".to_string()));
        assert!(!args.contains(&"-c:v".to_string()));
        assert!(!args.contains(&"ddagrab".to_string())); // Video Input
    }

    #[test]
    fn test_sanitize_preset() {
        // NVENC
        assert_eq!(FfmpegCommandBuilder::sanitize_preset("h264_nvenc", "balanced"), "p4");
        assert_eq!(FfmpegCommandBuilder::sanitize_preset("h264_nvenc", "speed"), "p2");
        assert_eq!(FfmpegCommandBuilder::sanitize_preset("h264_nvenc", "quality"), "p6");
        assert_eq!(FfmpegCommandBuilder::sanitize_preset("h264_nvenc", "p7"), "p7");

        // AMF
        assert_eq!(FfmpegCommandBuilder::sanitize_preset("h264_amf", "p4"), "balanced");
        assert_eq!(FfmpegCommandBuilder::sanitize_preset("h264_amf", "veryfast"), "speed");
        assert_eq!(FfmpegCommandBuilder::sanitize_preset("h264_amf", "quality"), "quality");

        // QSV
        assert_eq!(FfmpegCommandBuilder::sanitize_preset("h264_qsv", "p4"), "medium");
        assert_eq!(FfmpegCommandBuilder::sanitize_preset("h264_qsv", "speed"), "veryfast");
        assert_eq!(FfmpegCommandBuilder::sanitize_preset("h264_qsv", "quality"), "veryslow");

        // Software
        assert_eq!(FfmpegCommandBuilder::sanitize_preset("libx264", "p4"), "veryfast");
        assert_eq!(FfmpegCommandBuilder::sanitize_preset("libx264", "ultrafast"), "ultrafast");
    }
    #[test]
    fn test_format_bridging_qsv() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_video_codec("h264_qsv".to_string());
        let args = builder.build();
        
        // Should contain filter complex or vf with hwmap
        let has_filter = args.iter().any(|a| a.contains("hwmap=derive_device=qsv,format=qsv"));
        assert!(has_filter, "QSV should have hwmap filter");
    }

    #[test]
    fn test_format_bridging_software() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_video_codec("libx264".to_string());
        let args = builder.build();
        
        // Should contain filter complex or vf with hwdownload
        let has_filter = args.iter().any(|a| a.contains("hwdownload,format=nv12"));
        assert!(has_filter, "Software should have hwdownload filter");
    }

    #[test]
    fn test_format_bridging_nvenc_native() {
        let builder = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_video_codec("h264_nvenc".to_string());
        let args = builder.build();
        
        // Should NOT have bridging filters (native d3d11)
        let has_map = args.iter().any(|a| a.contains("hwmap"));
        let has_download = args.iter().any(|a| a.contains("hwdownload"));
        assert!(!has_map && !has_download, "NVENC should not have bridging filters");
    }
    #[test]
    fn test_pix_fmt_restoration() {
        // Case 1: Hardware Encoder (NVENC) -> Should use d3d11
        let builder_hw = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_video_codec("h264_nvenc".to_string());
        let args_hw = builder_hw.build();
        
        let pix_fmt_idx_hw = args_hw.iter().position(|r| r == "-pix_fmt").unwrap();
        assert_eq!(args_hw[pix_fmt_idx_hw + 1], "d3d11", "Hardware encoder should use d3d11 pix_fmt");

        // Case 2: Software Encoder (x264) -> Should use yuv420p
        let builder_sw = FfmpegCommandBuilder::new("output.mp4".to_string())
            .with_video_codec("libx264".to_string());
        let args_sw = builder_sw.build();
        
        let pix_fmt_idx_sw = args_sw.iter().position(|r| r == "-pix_fmt").unwrap();
        assert_eq!(args_sw[pix_fmt_idx_sw + 1], "yuv420p", "Software encoder should use yuv420p pix_fmt");
    }
}
