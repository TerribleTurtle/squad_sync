use tauri::{command, AppHandle, Manager};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MonitorInfo {
    pub id: u32, // We'll use index as ID for simplicity with ddagrab
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

#[command]
pub fn get_monitors(app: AppHandle) -> Result<Vec<MonitorInfo>, String> {
    let window = app.get_webview_window("main").ok_or("No main window")?;
    let monitors = window.available_monitors().map_err(|e| e.to_string())?;
    let primary = window.primary_monitor().ok().flatten();

    let mut result = Vec::new();
    for (index, monitor) in monitors.iter().enumerate() {
        let size = monitor.size();
        // Check if this is the primary monitor
        let is_primary = if let Some(p) = &primary {
            p.position().x == monitor.position().x && p.position().y == monitor.position().y
        } else {
            false
        };

        result.push(MonitorInfo {
            id: index as u32,
            name: monitor.name().map(|s| s.to_string()).unwrap_or_else(|| format!("Monitor {}", index + 1)),
            width: size.width,
            height: size.height,
            is_primary,
        });
    }

    Ok(result)
}
