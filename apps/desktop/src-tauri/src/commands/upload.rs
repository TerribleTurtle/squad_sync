use tauri::command;

#[command]
pub fn upload_clip() {
    log::info!("Upload clip command received");
}
