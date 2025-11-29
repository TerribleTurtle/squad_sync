use std::sync::Mutex;
use std::sync::mpsc::Sender;
use crate::config::AppConfig;

pub enum RecordingMessage {
    AudioData(Vec<u8>),
    Stop,
}

pub struct RecordingState {
    pub tx: Mutex<Option<Sender<RecordingMessage>>>,
    pub config: Mutex<AppConfig>,
}

impl RecordingState {
    pub fn new() -> Self {
        Self {
            tx: Mutex::new(None),
            // Config will be loaded properly in setup, but we need a default here
            config: Mutex::new(AppConfig::default()),
        }
    }
}
