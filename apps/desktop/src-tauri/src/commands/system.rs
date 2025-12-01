use tauri::command;

#[command]
pub fn get_system_info() {
    log::info!("Get system info command received");
}
