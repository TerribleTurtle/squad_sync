use tauri::command;

#[command]
pub fn create_clip() {
    println!("Create clip command received");
}
