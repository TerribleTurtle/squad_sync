use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;
use cpal::traits::{DeviceTrait, HostTrait};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub recording: RecordingConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecordingConfig {
    pub path: String,
    pub resolution: Option<String>,
    pub framerate: u32,
    pub bitrate: Option<String>,
    pub monitor_index: u32,
    pub encoder: String,
    pub audio_source: Option<String>,
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
}

impl Default for AppConfig {
    fn default() -> Self {
        // Auto-detect default microphone
        let audio_source = cpal::default_host()
            .default_input_device()
            .map(|d| d.name().unwrap_or_default())
            .filter(|n| !n.is_empty());

        Self {
            recording: RecordingConfig {
                path: String::new(), // Empty string implies default temp dir
                resolution: Some("native".to_string()),
                framerate: 60,
                bitrate: None,
                monitor_index: 0,
                encoder: "auto".to_string(),
                audio_source,
                audio_codec: None,
                video_preset: None,
                video_tune: None,
                video_profile: None,
                audio_bitrate: None,
            },
        }
    }
}


impl AppConfig {
    pub fn load(app: &AppHandle) -> Self {
        let config_path = get_config_path(app);
        
        if let Some(path) = &config_path {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(config) = toml::from_str(&content) {
                        return config;
                    }
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
