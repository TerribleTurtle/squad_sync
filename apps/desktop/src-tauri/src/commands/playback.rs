use tauri::{AppHandle, Manager, Runtime};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use std::fs;
use crate::state::RecordingState;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Recording {
    pub name: String,
    pub path: String,
    pub thumbnail_path: Option<String>,
    pub size: u64,
    pub created_at: u64,
    pub duration: Option<f64>, // Duration in seconds, optional for now
}

#[tauri::command]
pub async fn get_recordings(app: AppHandle) -> Result<Vec<Recording>, String> {
    let state = app.state::<RecordingState>();
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let output_path = if !config.recording.path.is_empty() {
        PathBuf::from(&config.recording.path)
    } else {
        app.path().video_dir().map_err(|e| e.to_string())?.join("SquadSync")
    };

    log::info!("Scanning for recordings in: {:?}", output_path);

    if !output_path.exists() {
        log::warn!("Output path does not exist: {:?}", output_path);
        return Ok(Vec::new());
    }

    let mut recordings = Vec::new();
    let entries = fs::read_dir(&output_path).map_err(|e| e.to_string())?;

    for entry in entries {
        if let Ok(entry) = entry {
            log::debug!("Found entry: {:?}", entry.path());
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    let ext_str = extension.to_string_lossy().to_lowercase();
                    if ["mp4", "mkv", "mov", "avi"].contains(&ext_str.as_str()) {
                        let metadata = fs::metadata(&path).map_err(|e| e.to_string())?;
                        let created_at = metadata.created()
                            .unwrap_or(SystemTime::now())
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        let thumbnail_path = output_path.join(".thumbnails").join(format!("{}.jpg", path.file_name().unwrap_or_default().to_string_lossy()));
                        let thumbnail_path_str = if thumbnail_path.exists() {
                            Some(thumbnail_path.to_string_lossy().to_string())
                        } else {
                            None
                        };

                        recordings.push(Recording {
                            name: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                            path: path.to_string_lossy().to_string(),
                            thumbnail_path: thumbnail_path_str,
                            size: metadata.len(),
                            created_at,
                            duration: None, // TODO: Extract duration if needed
                        });
                    }
                }
            }
        }
    }

    // Sort by created_at desc
    recordings.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(recordings)
}

#[tauri::command]
pub async fn delete_recording(path: String) -> Result<(), String> {
    let path_buf = PathBuf::from(&path);
    if path_buf.exists() {
        fs::remove_file(path_buf).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn rename_recording(path: String, new_name: String) -> Result<(), String> {
    let old_path = PathBuf::from(&path);
    if !old_path.exists() {
        return Err("File not found".to_string());
    }

    let parent = old_path.parent().ok_or("Invalid path")?;
    // Ensure new_name has the same extension if not provided
    let mut new_filename = new_name;
    if let Some(ext) = old_path.extension() {
        let ext_str = ext.to_string_lossy();
        if !new_filename.ends_with(&format!(".{}", ext_str)) {
            new_filename = format!("{}.{}", new_filename, ext_str);
        }
    }

    let new_path = parent.join(new_filename);
    fs::rename(old_path, new_path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn show_in_folder<R: Runtime>(_app: AppHandle<R>, path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("explorer")
            .args(["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .args(["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "linux")]
    {
        // Try dbus-send or xdg-open (xdg-open usually opens the file, not folder)
        // For now, just try opening the parent dir
        let path_buf = PathBuf::from(&path);
        if let Some(parent) = path_buf.parent() {
             use tauri_plugin_shell::ShellExt;
             app.shell().open(parent.to_string_lossy(), None).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn generate_thumbnail(app: AppHandle, path: String) -> Result<String, String> {
    let video_path = PathBuf::from(&path);
    if !video_path.exists() {
        return Err("Video file not found".to_string());
    }

    let parent = video_path.parent().ok_or("Invalid video path")?;
    let thumbnails_dir = parent.join(".thumbnails");
    
    if !thumbnails_dir.exists() {
        fs::create_dir_all(&thumbnails_dir).map_err(|e| e.to_string())?;
        
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            let _ = Command::new("attrib")
                .args(["+h", &thumbnails_dir.to_string_lossy()])
                .output();
        }
    }

    let thumbnail_filename = format!("{}.jpg", video_path.file_name().unwrap_or_default().to_string_lossy());
    let thumbnail_path = thumbnails_dir.join(&thumbnail_filename);

    if thumbnail_path.exists() {
        return Ok(thumbnail_path.to_string_lossy().to_string());
    }

    let ffmpeg_path = crate::ffmpeg::utils::get_sidecar_path(&app, "ffmpeg")
        .map_err(|e| format!("FFmpeg not found: {}", e))?;

    let mut cmd = std::process::Command::new(ffmpeg_path);
    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;
    #[cfg(target_os = "windows")]
    cmd.creation_flags(0x08000000);

    log::info!("Generating thumbnail for: {} -> {}", path, thumbnail_path.to_string_lossy());

    let output = cmd
        .args([
            "-y",
            "-ss", "00:00:01",
            "-i", &path,
            "-vframes", "1",
            "-q:v", "2",
            thumbnail_path.to_string_lossy().as_ref()
        ])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        log::info!("Thumbnail generated successfully");
        Ok(thumbnail_path.to_string_lossy().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        log::error!("Failed to generate thumbnail: {}", stderr);
        Err(format!("Failed to generate thumbnail: {}", stderr))
    }
}

#[tauri::command]
pub async fn open_file<R: Runtime>(app: AppHandle<R>, path: String) -> Result<(), String> {
    use tauri_plugin_shell::ShellExt;
    app.shell().open(path, None).map_err(|e| e.to_string())?;
    Ok(())
}
