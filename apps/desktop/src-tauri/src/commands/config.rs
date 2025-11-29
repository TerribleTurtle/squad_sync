use tauri::{command, AppHandle, State};
use crate::state::RecordingState;
use crate::config::AppConfig;

#[command]
pub fn get_config(state: State<'_, RecordingState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[command]
pub fn update_config(app: AppHandle, state: State<'_, RecordingState>, new_config: AppConfig) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    *config = new_config.clone();
    
    // Persist to disk
    config.save(&app)?;
    
    Ok(())
}
