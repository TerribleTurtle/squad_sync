use std::sync::Mutex;
use std::sync::mpsc::Sender;
use crate::config::AppConfig;

pub enum RecordingMessage {
    AudioData(Vec<u8>),
    Stop,
}

use crate::ntp::NtpManager;
use std::sync::Arc;

pub struct RecordingState {
    pub tx: Mutex<Option<Sender<RecordingMessage>>>,
    pub join_handle: Mutex<Option<std::thread::JoinHandle<()>>>,
    pub config: Mutex<AppConfig>,
    pub last_clip_timestamp: Mutex<Option<std::time::Instant>>,
    pub ntp_manager: Arc<NtpManager>,
}

impl RecordingState {
    pub fn new() -> Self {
        Self {
            tx: Mutex::new(None),
            join_handle: Mutex::new(None),
            // Config will be loaded properly in setup, but we need a default here
            config: Mutex::new(AppConfig::default()),
            last_clip_timestamp: Mutex::new(None),
            ntp_manager: Arc::new(NtpManager::new()),
        }
    }
}
