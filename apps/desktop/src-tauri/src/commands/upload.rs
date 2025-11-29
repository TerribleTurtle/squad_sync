use tauri::command;

#[command]
pub fn upload_clip() {
    println!("Upload clip command received");
}
