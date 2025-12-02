use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;
use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub recording: RecordingConfig,
    #[serde(default)]
    pub user: UserConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecordingConfig {
    pub path: String,
    #[serde(default = "default_temp_path")]
    pub temp_path: String,
    pub resolution: Option<String>,
    pub framerate: u32,
    pub bitrate: Option<String>,
    #[serde(default = "default_buffer_duration")]
    pub buffer_duration: u32,
    #[serde(default = "default_segment_time")]
    pub segment_time: u32,
    pub monitor_index: u32,
    pub encoder: String,
    pub audio_source: Option<String>,
    pub system_audio_device: Option<String>,
    pub audio_codec: Option<String>,
    
    // Advanced Overrides (Hidden from default config)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_preset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_tune: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_bitrate: Option<String>,
    #[serde(default = "default_buffer_retention_seconds")]
    pub buffer_retention_seconds: u32,
    #[serde(default = "default_audio_backend")]
    pub audio_backend: String, // "cpal" or "dshow"
}

fn default_audio_backend() -> String {
    "cpal".to_string()
}

fn default_temp_path() -> String {
    if let Some(mut path) = dirs::data_local_dir() {
        path.push("SquadSync");
        path.push("Buffer");
        return path.to_string_lossy().to_string();
    }
    // Fallback if dirs fails (unlikely on Windows)
    "C:\\SquadSync_Buffer".to_string()
}

fn default_buffer_duration() -> u32 {
    60
}

fn default_segment_time() -> u32 {
    15
}

fn default_buffer_retention_seconds() -> u32 {
    480 // 8 minutes
}

impl Default for AppConfig {
    fn default() -> Self {
        // Auto-detect default microphone
        let audio_source = cpal::default_host()
            .default_input_device()
            .map(|d| d.name().unwrap_or_default())
            .filter(|n| !n.is_empty());

        // Auto-detect default speaker
        let system_audio_device = cpal::default_host()
            .default_output_device()
            .map(|d| d.name().unwrap_or_default())
            .filter(|n| !n.is_empty());

        Self {
            recording: RecordingConfig {
                path: String::new(),
                temp_path: default_temp_path(),
                resolution: Some("1920x1080".to_string()),
                framerate: 60,
                bitrate: None,
                buffer_duration: 60, // 1 minute default buffer
                segment_time: 15,     // 15 second segments
                monitor_index: 0,
                encoder: "auto".to_string(),
                audio_source,
                system_audio_device,
                audio_codec: None,
                video_preset: None,
                video_tune: None,
                video_profile: None,
                audio_bitrate: None,
                buffer_retention_seconds: 300,
                audio_backend: "cpal".to_string(),
            },
            user: UserConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct UserConfig {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
}


impl AppConfig {
    pub fn load(app: &AppHandle) -> Self {
        let config_path = get_config_path(app);
        
        if let Some(path) = &config_path {
            if path.exists() {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        match toml::from_str(&content) {
                            Ok(config) => return config,
                            Err(e) => log::error!("Failed to parse config file: {}", e),
                        }
                    },
                    Err(e) => log::error!("Failed to read config file: {}", e),
                }
            }
        }

        // Return default if load fails or file doesn't exist
        let default_config = Self::default();
        // Try to save the default config so the user has a file to edit
        if let Some(path) = &config_path {
             let _ = default_config.save_to_path(path);
        }
        
        default_config
    }

    pub fn save(&self, app: &AppHandle) -> Result<(), String> {
        let config_path = get_config_path(app).ok_or("Could not resolve config path")?;
        self.save_to_path(&config_path)
    }

    fn save_to_path(&self, path: &PathBuf) -> Result<(), String> {
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        
        fs::write(path, content).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn get_config_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|p| p.join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.recording.framerate, 60);
        assert_eq!(config.recording.bitrate, None);
        assert_eq!(config.recording.encoder, "auto");
    }

    #[test]
    fn test_serialization() {
        let config = AppConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&toml).unwrap();
        
        assert_eq!(config.recording.framerate, deserialized.recording.framerate);
        assert_eq!(config.recording.bitrate, deserialized.recording.bitrate);
    }
}
