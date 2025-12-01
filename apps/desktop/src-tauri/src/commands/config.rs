use tauri::{command, AppHandle, State};
use crate::state::{RecordingState, RecordingMessage};
use crate::config::AppConfig;
use crate::ffmpeg::process::start_recording_process;

#[command]
pub fn get_config(state: State<'_, RecordingState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[command]
pub async fn update_config(app: AppHandle, state: State<'_, RecordingState>, new_config: AppConfig) -> Result<(), String> {
    let mut was_recording = false;
    let mut handle_to_join = None;

    // 1. Check if recording is active and stop it
    {
        let mut tx_guard = state.tx.lock().map_err(|e| e.to_string())?;
        if let Some(tx) = tx_guard.take() {
            was_recording = true;
            log::info!("Settings changed while recording. Stopping to apply changes...");
            let _ = tx.send(RecordingMessage::Stop);
            
            // Take handle to join
            let mut handle_guard = state.join_handle.lock().map_err(|e| e.to_string())?;
            handle_to_join = handle_guard.take();
        }
    }

    // 1b. Wait for cleanup if needed
    if let Some(handle) = handle_to_join {
        log::info!("Waiting for previous recording to cleanup...");
        if let Err(_) = handle.join() {
            log::error!("Failed to join previous recording thread");
        }
        log::info!("Previous recording cleaned up.");
    }

    // 2. Update Config
    {
        let mut config = state.config.lock().map_err(|e| e.to_string())?;
        *config = new_config.clone();
        config.save(&app)?;
    }

    // 3. Restart if it was recording
    if was_recording {
        log::info!("Restarting recording with new settings...");
        // Small delay to ensure previous process cleanup if needed, though not strictly required if we spawn new one.
        // But let's just spawn.
        match start_recording_process(&app).await {
            Ok((tx, handle)) => {
                let mut tx_guard = state.tx.lock().map_err(|e| e.to_string())?;
                *tx_guard = Some(tx);
                
                let mut handle_guard = state.join_handle.lock().map_err(|e| e.to_string())?;
                *handle_guard = Some(handle);

                log::info!("Recording restarted successfully.");
            }
            Err(e) => {
                log::error!("Failed to restart recording: {}", e);
                return Err(format!("Settings saved, but failed to restart recording: {}", e));
            }
        }
    }
    
    Ok(())
}
