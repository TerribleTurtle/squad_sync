use tauri::command;

#[command]
pub fn get_system_info() {
    println!("Get system info command received");
}
