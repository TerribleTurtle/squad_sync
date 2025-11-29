use serde::Serialize;
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO Error: {0}")]
    Io(#[from] io::Error),

    #[error("FFmpeg Error: {0}")]
    Ffmpeg(String),

    #[error("Audio Error: {0}")]
    Audio(String),

    #[error("Config Error: {0}")]
    Config(String),
    
    #[error("State Error: {0}")]
    State(String),
}

// Allow serializing errors to send to frontend
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

// Helper to convert strings to AppError easily
impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Ffmpeg(s) // Default to generic/ffmpeg error for string conversions
    }
}
