use tauri::{command, AppHandle, State};
use crate::state::{RecordingState, RecordingMessage};
use crate::ffmpeg::process::start_recording_process;

#[command]
pub fn start_recording(app: AppHandle, state: State<'_, RecordingState>) -> Result<(), String> {
    println!("Start recording command received");
    
    let mut tx_guard = state.tx.lock().map_err(|e| e.to_string())?;
    
    if tx_guard.is_some() {
        return Err("Recording is already in progress".to_string());
    }

    match start_recording_process(&app) {
        Ok(tx) => {
            *tx_guard = Some(tx);
            Ok(())
        }
        Err(e) => Err(format!("Failed to start recording process: {}", e)),
    }
}

#[command]
pub fn stop_recording(state: State<'_, RecordingState>) -> Result<(), String> {
    println!("Stop recording command received");
    
    let mut tx_guard = state.tx.lock().map_err(|e| e.to_string())?;
    
    if let Some(tx) = tx_guard.take() {
        // Send Stop signal
        match tx.send(RecordingMessage::Stop) {
            Ok(_) => println!("Sent Stop signal to recording thread"),
            Err(e) => eprintln!("Failed to send Stop signal: {}", e),
        }
    } else {
        println!("No recording in progress to stop");
    }
    
    Ok(())
}
