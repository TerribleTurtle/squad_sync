use tauri::{command, AppHandle, Manager};
use crate::state::{RecordingState, RecordingMessage};
use crate::ffmpeg::process::start_recording_process;

#[command]
pub async fn enable_replay(app: AppHandle) -> Result<(), String> {
    log::info!("Enable Replay command received");
    
    // Release the lock BEFORE awaiting start_recording_process
    {
        let state = app.state::<RecordingState>();
        let tx_guard = state.tx.lock().map_err(|e| e.to_string())?;
        if tx_guard.is_some() {
            return Err("Replay Buffer already active".to_string());
        }
    } // Lock released here

    // Now await the process start
    match start_recording_process(&app).await {
        Ok((tx, handle)) => {
            let state = app.state::<RecordingState>();
            
            // Store sender
            {
                let mut tx_guard = state.tx.lock().map_err(|e| e.to_string())?;
                *tx_guard = Some(tx);
            }
            // Store handle
            {
                let mut handle_guard = state.join_handle.lock().map_err(|e| e.to_string())?;
                *handle_guard = Some(handle);
            }
            
            Ok(())
        }
        Err(e) => Err(e),
    }
}

#[command]
pub async fn disable_replay(app: AppHandle) -> Result<(), String> {
    log::info!("Disable Replay command received");
    let state = app.state::<RecordingState>();
    
    // Send Stop signal
    {
        let mut tx_guard = state.tx.lock().map_err(|e| e.to_string())?;
        if let Some(tx) = tx_guard.take() {
            match tx.send(RecordingMessage::Stop) {
                Ok(_) => log::info!("Sent Stop signal to recording thread"),
                Err(e) => log::error!("Failed to send Stop signal: {}", e),
            }
        } else {
            return Err("Replay Buffer not active".to_string());
        }
    }

    // Take handle to join outside lock
    let handle_to_join = {
        let mut handle_guard = state.join_handle.lock().map_err(|e| e.to_string())?;
        handle_guard.take()
    };

    if let Some(handle) = handle_to_join {
        log::info!("Waiting for recording thread to finish cleanup...");
        if handle.join().is_err() {
            log::error!("Failed to join recording thread");
        }
        log::info!("Recording thread joined successfully");
    }

    Ok(())
}
