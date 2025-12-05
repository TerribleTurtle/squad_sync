
fn find_primary_monitor(window: &tauri::WebviewWindow, monitors: &[tauri::Monitor]) -> (Option<tauri::Monitor>, Option<u32>) {
    if let Ok(Some(primary)) = window.primary_monitor() {
        let primary_index = monitors.iter().position(|m| 
            m.position().x == primary.position().x && m.position().y == primary.position().y
        ).unwrap_or(0) as u32;
        
        println!("Primary Monitor found at index {}.", primary_index);
        (Some(primary), Some(primary_index))
    } else {
        println!("No Primary Monitor found. Defaulting to index 0.");
        (monitors.first().cloned(), Some(0))
    }
}
