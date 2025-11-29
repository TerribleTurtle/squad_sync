use std::sync::Mutex;
use tauri_plugin_shell::process::CommandChild;

pub struct RecordingState {
    pub child: Mutex<Option<CommandChild>>,
}

impl RecordingState {
    pub fn new() -> Self {
        Self {
            child: Mutex::new(None),
        }
    }
}
