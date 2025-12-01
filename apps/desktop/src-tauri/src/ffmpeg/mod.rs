//! FFmpeg Module
//! 
//! This module handles all video recording and processing functionality.
//! 
//! # Architecture
//! 
//! * `process`: High-level orchestration. Starts the recording session, manages configuration, and handles the temp buffer.
//! * `session`: Manages the actual FFmpeg child process, including spawning, monitoring, and cleanup.
//! * `commands`: Builder pattern for constructing complex FFmpeg CLI arguments.
//! * `monitor`: Parses FFmpeg stderr output to track recording status (bitrate, time, etc.).
//! * `encoder`: Handles hardware encoder detection and selection.
//! * `utils`: Shared utility functions.

pub mod process;
pub mod commands;
pub mod encoder;
pub mod monitor;
pub mod session;
pub mod utils;
