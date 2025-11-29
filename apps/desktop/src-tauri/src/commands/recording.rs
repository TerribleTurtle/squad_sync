use tauri::{command, AppHandle, State};
use crate::state::RecordingState;
use crate::ffmpeg::process::spawn_ffmpeg;

#[command]
pub fn start_recording(app: AppHandle, state: State<'_, RecordingState>) -> Result<(), String> {
    println!("Start recording command received");
    
    let mut child_guard = state.child.lock().map_err(|e| e.to_string())?;
    
    if child_guard.is_some() {
        return Err("Recording is already in progress".to_string());
    }

    match spawn_ffmpeg(&app) {
        Ok(child) => {
            *child_guard = Some(child);
            Ok(())
        }
        Err(e) => Err(format!("Failed to spawn FFmpeg: {}", e)),
    }
}

#[command]
pub fn stop_recording(state: State<'_, RecordingState>) -> Result<(), String> {
    println!("Stop recording command received");
    
    let mut child_guard = state.child.lock().map_err(|e| e.to_string())?;
    
    if let Some(child) = child_guard.take() {
        child.kill().map_err(|e| e.to_string())?;
        println!("FFmpeg process killed");
    } else {
        println!("No recording in progress to stop");
    }
    
    Ok(())
}
