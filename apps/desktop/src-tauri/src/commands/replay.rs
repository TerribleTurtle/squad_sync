use tauri::{command, AppHandle, Manager};
use crate::state::RecordingState;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

#[command]
pub async fn save_replay(app: AppHandle) -> Result<String, String> {
    save_replay_impl(&app).await
}

pub async fn save_replay_impl(app: &AppHandle) -> Result<String, String> {
    println!("Save Replay triggered");
    
    let state = app.state::<RecordingState>();
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    
    // 1. Check if Replay is Active (by checking if temp path exists and has segments)
    // Actually, we should check if the process is running, but checking the buffer dir is a good proxy
    let temp_path_str = config.recording.temp_path.replace("%TEMP%", &std::env::temp_dir().to_string_lossy());
    let buffer_dir = PathBuf::from(&temp_path_str);
    let playlist_path = buffer_dir.join("buffer.m3u8");

    // Clean up old .mp4 segments if they exist (from previous version)
    // This is a one-time cleanup or just good hygiene
    if let Ok(entries) = fs::read_dir(&buffer_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "mp4" && path.file_name().unwrap().to_string_lossy().starts_with("clip_") {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }

    if !playlist_path.exists() {
        return Err("Replay buffer not active (playlist not found)".to_string());
    }

    // 2. Calculate Duration to Save
    let mut last_clip_ts = state.last_clip_timestamp.lock().map_err(|e| e.to_string())?;
    let now = Instant::now();
    
    let duration_to_save = if let Some(ts) = *last_clip_ts {
        let elapsed = now.duration_since(ts).as_secs() as u32;
        if elapsed < config.recording.buffer_duration {
            elapsed
        } else {
            config.recording.buffer_duration
        }
    } else {
        config.recording.buffer_duration
    };
    
    // Update timestamp
    *last_clip_ts = Some(now);

    println!("Saving last {} seconds", duration_to_save);

    // 3. Parse Playlist
    let playlist_content = fs::read_to_string(&playlist_path).map_err(|e| e.to_string())?;
    let segments = parse_m3u8(&playlist_content);
    
    if segments.is_empty() {
        return Err("No segments in buffer".to_string());
    }

    // 4. Select Segments
    // Simple logic: Take enough segments from the end to cover the duration
    // We assume segments are roughly 'segment_time' length
    let segment_time = config.recording.segment_time;
    let segments_needed = (duration_to_save as f32 / segment_time as f32).ceil() as usize;
    let segments_to_stitch = if segments_needed > segments.len() {
        &segments[..]
    } else {
        &segments[segments.len() - segments_needed..]
    };

    if segments_to_stitch.is_empty() {
        return Err("No segments selected".to_string());
    }

    // 5. Create Concat List
    let concat_list_path = buffer_dir.join("concat_list.txt");
    let mut concat_content = String::new();
    for segment in segments_to_stitch {
        // FFmpeg concat demuxer requires "file 'path'"
        // Paths should be absolute or relative to the list file
        concat_content.push_str(&format!("file '{}'\n", buffer_dir.join(segment).to_string_lossy().replace("\\", "/")));
    }
    fs::write(&concat_list_path, concat_content).map_err(|e| e.to_string())?;

    // 6. Spawn Stitching Process
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let output_filename = format!("Replay_{}.mp4", timestamp);
    
    // Determine output directory (User's recording path or Video dir)
    let output_dir = if !config.recording.path.is_empty() {
        PathBuf::from(&config.recording.path)
    } else {
        app.path().video_dir().map_err(|e| e.to_string())?.join("SquadSync")
    };
    
    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
    }
    
    let output_path = output_dir.join(&output_filename);

    println!("Stitching to: {:?}", output_path);

    // Run FFmpeg
    // ffmpeg -f concat -safe 0 -i concat_list.txt -c copy output.mp4
    let status = Command::new("ffmpeg")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(&concat_list_path)
        .arg("-c")
        .arg("copy")
        .arg("-y") // Overwrite if exists (unlikely with timestamp)
        .arg(&output_path)
        .status()
        .map_err(|e| format!("Failed to execute ffmpeg: {}", e))?;

    if status.success() {
        Ok(output_path.to_string_lossy().to_string())
    } else {
        Err("Stitching failed".to_string())
    }
}

fn parse_m3u8(content: &str) -> Vec<String> {
    let mut segments = Vec::new();
    for line in content.lines() {
        if !line.starts_with('#') && !line.trim().is_empty() {
            segments.push(line.trim().to_string());
        }
    }
    segments
}
