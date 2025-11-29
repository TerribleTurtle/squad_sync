use std::sync::Mutex;
use tauri_plugin_shell::process::CommandChild;
use crate::config::AppConfig;

pub struct RecordingState {
    pub child: Mutex<Option<CommandChild>>,
    pub config: Mutex<AppConfig>,
}

impl RecordingState {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
            // Config will be loaded properly in setup, but we need a default here
            config: Mutex::new(AppConfig::default()),
        }
    }
}
