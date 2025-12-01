use tauri::command;

#[command]
pub fn create_clip() {
    log::info!("Create clip command received");
}
