use tauri::{command, AppHandle, Manager};
use crate::state::RecordingState;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use regex::Regex;
use serde::Deserialize;
use chrono::{DateTime, Local, TimeZone, Duration};

#[derive(Deserialize)]
struct RecordingMetadata {
    video_start_time: u64,
    audio_start_time: u64,
}

#[command]
pub async fn save_replay(app: AppHandle) -> Result<String, String> {
    save_replay_impl(&app).await
}

pub async fn save_replay_impl(app: &AppHandle) -> Result<String, String> {
    log::info!("Save Replay triggered (Time-Based)");
    let start_process_time = std::time::Instant::now();

    // Wait for FFmpeg to flush recent packets
    std::thread::sleep(std::time::Duration::from_millis(crate::constants::REPLAY_FLUSH_WAIT_MS));

    let state = app.state::<RecordingState>();
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let buffer_dir = PathBuf::from(&config.recording.temp_path);

    // 1. Cleanup Old Segments
    if let Err(e) = cleanup_buffer(&buffer_dir, config.recording.buffer_retention_seconds) {
        log::warn!("Warning: Failed to cleanup buffer: {}", e);
    }

    // 2. Determine Trigger Time (NTP -> Local)
    // Get NTP time (authoritative)
    let ntp_time_ms = state.ntp_manager.get_ntp_time_ms();
    let ntp_offset = state.ntp_manager.get_offset();
    
    // Convert back to Local System Time for file searching
    // TriggerTime_Local = TriggerTime_NTP - Offset
    // (Because FileTime = SystemTime)
    let trigger_time_ms = if ntp_offset >= 0 {
        ntp_time_ms.saturating_sub(ntp_offset as u64)
    } else {
        ntp_time_ms + ((-ntp_offset) as u64)
    };

    let trigger_time = std::time::UNIX_EPOCH + std::time::Duration::from_millis(trigger_time_ms);
    let trigger_datetime: DateTime<Local> = trigger_time.into();
    
    log::info!("Trigger Time: {} (NTP: {}, Offset: {})", trigger_datetime, ntp_time_ms, ntp_offset);

    // 3. Define Time Range
    let duration_sec = config.recording.buffer_duration as i64;
    let start_time = trigger_datetime - Duration::seconds(duration_sec);
    let end_time = trigger_datetime;

    log::info!("Searching for segments between {} and {}", start_time, end_time);

    // 4. Find Segments
    let video_segments = find_segments_by_time(&buffer_dir, "video_", start_time, end_time)?;
    if video_segments.is_empty() {
        return Err("No video segments found for the requested time range".to_string());
    }

    // Audio is optional
    let audio_segments = find_segments_by_time(&buffer_dir, "audio_", start_time, end_time).unwrap_or_default();
    let has_audio = !audio_segments.is_empty();

    // 4b. Smart Wait (Ensure active segment is finished)
    // If the last segment is very recent, it might still be writing.
    // We wait until a NEWER segment appears, confirming the previous one is closed.
    if let Some(last_video) = video_segments.last() {
        wait_for_segment_completion(last_video, &buffer_dir);
    }
    if let Some(last_audio) = audio_segments.last() {
        wait_for_segment_completion(last_audio, &buffer_dir);
    }

    // 5. Setup Temp Dir
    let timestamp_str = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let stitch_temp_dir = buffer_dir.join(format!("stitch_{}", timestamp_str));
    fs::create_dir_all(&stitch_temp_dir).map_err(|e| e.to_string())?;

    // 6. Probe Start Time (Precision)
    // We need the EXACT start time of the first video segment to calculate trim
    let first_video_path = &video_segments[0];
    let first_video_start_ms = probe_start_time(app, first_video_path).map_err(|e| format!("Failed to probe video start: {}", e))?;
    
    // Calculate Trim Start
    // Target Start = Trigger - Duration
    // Actual Start = First Segment Start
    // Trim = Target Start - Actual Start
    // If Target Start < Actual Start, we can't trim (we missed the start), so Trim = 0.
    
    let target_start_ms = trigger_time_ms - (duration_sec as u64 * 1000);
    let trim_start_sec = if target_start_ms > first_video_start_ms {
        (target_start_ms - first_video_start_ms) as f64 / 1000.0
    } else {
        0.0
    };

    log::info!("Precision Trim: Target={}, Actual={}, Trim={:.3}s", target_start_ms, first_video_start_ms, trim_start_sec);

    // 7. Stitch Video
    let temp_video_path = stitch_temp_dir.join("temp_video.mp4");
    stitch_segments(app, &video_segments, &stitch_temp_dir, &temp_video_path)?;

    // 8. Stitch Audio
    let temp_audio_path = stitch_temp_dir.join("temp_audio.mp4");
    if has_audio {
        stitch_segments(app, &audio_segments, &stitch_temp_dir, &temp_audio_path)?;
    }

    // 9. Merge & Trim
    let output_filename = format!("Replay_{}.mp4", timestamp_str);
    let output_dir = if !config.recording.path.is_empty() {
        PathBuf::from(&config.recording.path)
    } else {
        app.path().video_dir().map_err(|e| e.to_string())?.join("SquadSync")
    };

    if !output_dir.exists() {
        fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;
    }
    let output_path = output_dir.join(&output_filename);

    let ffmpeg_path = crate::ffmpeg::utils::get_sidecar_path(app, "ffmpeg")
        .map_err(|e| format!("FFmpeg not found: {}", e))?;

    let mut cmd = Command::new(ffmpeg_path);
    cmd.arg("-y");

    // Input Video
    cmd.arg("-ss").arg(trim_start_sec.to_string());
    cmd.arg("-i").arg(&temp_video_path);

    if has_audio {
        // Audio might need a different trim if it started at a different time?
        // Ideally, if we use wallclock timestamps, they should align.
        // But we are stitching separate files.
        // Let's assume they align roughly or use the same trim for now.
        // For frame-perfect audio, we should probe audio start too.
        
        let first_audio_path = &audio_segments[0];
        if let Ok(first_audio_start_ms) = probe_start_time(app, first_audio_path) {
             let audio_trim_sec = if target_start_ms > first_audio_start_ms {
                (target_start_ms - first_audio_start_ms) as f64 / 1000.0
            } else {
                0.0
            };
            cmd.arg("-ss").arg(audio_trim_sec.to_string());
        } else {
            // Fallback
            cmd.arg("-ss").arg(trim_start_sec.to_string());
        }
        
        cmd.arg("-i").arg(&temp_audio_path);
    }

    // Map & Encode
    cmd.arg("-map").arg("0:v");
    if has_audio {
        cmd.arg("-map").arg("1:a");
        cmd.arg("-c:a").arg("aac");
        cmd.arg("-b:a").arg("192k");
    }

    cmd.arg("-c:v").arg("copy");
    cmd.arg("-t").arg(duration_sec.to_string());
    cmd.arg("-movflags").arg("+faststart");
    cmd.arg(&output_path);

    let status = cmd.status().map_err(|e| format!("Merge failed: {}", e))?;

    // Cleanup
    let _ = fs::remove_dir_all(&stitch_temp_dir);

    if status.success() {
        Ok(output_path.to_string_lossy().to_string())
    } else {
        Err("FFmpeg merge process failed".to_string())
    }
}

fn find_segments_by_time(
    dir: &PathBuf, 
    prefix: &str, 
    start: DateTime<Local>, 
    end: DateTime<Local>
) -> Result<Vec<PathBuf>, String> {
    let mut segments = Vec::new();
    // Regex: prefix + YYYYMMDDHHMMSS + .mkv
    // e.g. video_20251201071802.mkv
    let pattern = format!(r"^{}(\d{{14}})\.mkv$", prefix); 
    let re = Regex::new(&pattern).map_err(|e| e.to_string())?;

    if !dir.exists() {
        return Ok(vec![]);
    }

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(caps) = re.captures(fname) {
                if let Some(ts_str) = caps.get(1) {
                    // Parse timestamp
                    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(ts_str.as_str(), "%Y%m%d%H%M%S") {
                        if let Some(ts) = Local.from_local_datetime(&naive).single() {
                            // Check overlap
                            // Segment covers [ts, ts + 2s] roughly (actually 15s in config, but logic handles overlap)
                            // We want segments where End > Start_Req AND Start < End_Req
                            // ffprobe or config would be better for duration, but let's assume 15s based on logs
                            let seg_start = ts;
                            let seg_end = ts + Duration::seconds(15); 

                            if seg_end > start && seg_start < end {
                                segments.push((path, seg_start));
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by time
    segments.sort_by_key(|k| k.1);

    if segments.is_empty() {
        log::warn!("No segments found for range: {} to {} (Prefix: {})", start, end, prefix);
        // Debug scan
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(fname) = entry.path().file_name().and_then(|n| n.to_str()) {
                    if fname.starts_with(prefix) && fname.ends_with(".mkv") {
                         log::warn!("  Candidate ignored (Time mismatch?): {}", fname);
                    }
                }
            }
        }
    }

    Ok(segments.into_iter().map(|(p, _)| p).collect())
}

fn probe_start_time(app: &AppHandle, path: &PathBuf) -> Result<u64, String> {
    // 1. Parse Filename for approximate Epoch (Fallback & Validation)
    let fname = path.file_name().and_then(|n| n.to_str()).ok_or("Invalid filename")?;
    let re = Regex::new(r"(\d{14})\.mkv").map_err(|e| e.to_string())?;
    let filename_epoch_ms = if let Some(caps) = re.captures(fname) {
        if let Some(ts_str) = caps.get(1) {
            if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(ts_str.as_str(), "%Y%m%d%H%M%S") {
                if let Some(ts) = Local.from_local_datetime(&naive).single() {
                     Some(ts.timestamp_millis() as u64)
                } else { None }
            } else { None }
        } else { None }
    } else { None };

    // 2. Probe with ffprobe
    let ffprobe_path = crate::ffmpeg::utils::get_sidecar_path(app, "ffprobe")
        .map_err(|e| format!("FFprobe not found: {}", e))?;

    let output = Command::new(ffprobe_path)
        .args(&[
            "-v", "error",
            "-show_entries", "format=start_time",
            "-of", "default=noprint_wrappers=1:nokey=1",
            path.to_string_lossy().as_ref()
        ])
        .output()
        .map_err(|e| e.to_string())?;

    let mut probe_epoch_ms = None;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(start_sec) = stdout.trim().parse::<f64>() {
            probe_epoch_ms = Some((start_sec * 1000.0) as u64);
        }
    }

    // 3. Decision Logic
    match (filename_epoch_ms, probe_epoch_ms) {
        (Some(fn_ms), Some(pr_ms)) => {
            // If probe is "reasonable" (e.g. > year 2000), use it.
            // 946684800000 is year 2000.
            if pr_ms > 946684800000 {
                // It's likely Epoch. Use it for precision.
                // Sanity check: is it close to filename? (within 1 hour?)
                // If it's wildly different, trust filename?
                // Actually, if we used wallclock, it should be very close.
                Ok(pr_ms)
            } else {
                // Probe is likely relative (0 or small).
                // We want Epoch.
                // But we wanted sub-second precision!
                // If probe is 0.0, we lose sub-second.
                // If probe is 0.123, maybe we can add it to filename?
                // Risk: Filename is integer second. We don't know if it rounded down or up?
                // strftime %S usually truncates.
                // So Filename Time <= Actual Time.
                // If probe is relative, it might be offset from Filename Time?
                // No, relative usually means "from start of file".
                // Let's just fallback to filename for safety to fix the "empty clip" bug first.
                log::warn!("Probe returned non-Epoch time ({}). Falling back to filename time ({}).", pr_ms, fn_ms);
                Ok(fn_ms)
            }
        },
        (Some(fn_ms), None) => {
            log::warn!("Probe failed. Falling back to filename time.");
            Ok(fn_ms)
        },
        (None, Some(pr_ms)) => {
            // No filename time? Trust probe if it looks like Epoch.
             if pr_ms > 946684800000 {
                 Ok(pr_ms)
             } else {
                 Err("Probe returned relative time and filename parsing failed.".to_string())
             }
        },
        (None, None) => Err("Failed to determine start time from both probe and filename.".to_string())
    }
}

fn stitch_segments(app: &AppHandle, segments: &[PathBuf], temp_dir: &PathBuf, output_path: &PathBuf) -> Result<(), String> {
    let list_path = temp_dir.join("concat_list.txt");
    let mut content = String::new();
    
    for seg in segments {
        // Copy to temp dir to avoid file locking issues or path issues?
        // Or just reference absolute path. FFmpeg supports absolute paths in concat list.
        // But Windows paths need escaping.
        let path_str = seg.to_string_lossy().replace("\\", "/");
        content.push_str(&format!("file '{}'\n", path_str));
    }
    
    fs::write(&list_path, content).map_err(|e| e.to_string())?;

    let ffmpeg_path = crate::ffmpeg::utils::get_sidecar_path(app, "ffmpeg")
        .map_err(|e| format!("FFmpeg not found: {}", e))?;

    let status = Command::new(ffmpeg_path)
        .arg("-f").arg("concat")
        .arg("-safe").arg("0")
        .arg("-i").arg(&list_path)
        .arg("-c").arg("copy")
        .arg("-y")
        .arg(output_path)
        .status()
        .map_err(|e| e.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err("Stitch failed".to_string())
    }
}

pub fn cleanup_buffer(buffer_dir: &PathBuf, retention_seconds: u32) -> std::io::Result<()> {
    if !buffer_dir.exists() { return Ok(()); }
    
    let now = Local::now();
    let retention = Duration::seconds(retention_seconds as i64);
    
    // Regex for new pattern
    let re = Regex::new(r"(video|audio)_(\d{14})\.mkv").unwrap();

    for entry in fs::read_dir(buffer_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
            if let Some(caps) = re.captures(fname) {
                if let Some(ts_str) = caps.get(2) {
                    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(ts_str.as_str(), "%Y%m%d%H%M%S") {
                        if let Some(ts) = Local.from_local_datetime(&naive).single() {
                             if now - ts > retention {
                                 let _ = fs::remove_file(path);
                             }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn wait_for_segment_completion(segment_path: &PathBuf, buffer_dir: &PathBuf) {
    // Extract timestamp from segment
    let re = Regex::new(r"(\d{14})\.mkv").unwrap();
    let fname = segment_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    
    let current_ts = if let Some(caps) = re.captures(fname) {
        if let Some(ts_str) = caps.get(1) {
            chrono::NaiveDateTime::parse_from_str(ts_str.as_str(), "%Y%m%d%H%M%S").ok()
        } else { None }
    } else { None };

    if let Some(current_naive) = current_ts {
        let current_time = Local.from_local_datetime(&current_naive).single().unwrap();
        
        // Wait loop
        let max_retries = 10; // 5 seconds (10 * 500ms)
        for _ in 0..max_retries {
            // Check if a newer file exists
            let mut newer_found = false;
            if let Ok(entries) = fs::read_dir(buffer_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if let Some(caps) = re.captures(name) {
                            if let Some(ts_str) = caps.get(1) {
                                if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(ts_str.as_str(), "%Y%m%d%H%M%S") {
                                    if let Some(ts) = Local.from_local_datetime(&naive).single() {
                                        if ts > current_time {
                                            newer_found = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if newer_found {
                log::info!("Newer segment found. Active segment {} considered safe.", fname);
                return;
            }

            // Also check if file hasn't been modified for a while (fallback if recording stopped)
            if let Ok(meta) = fs::metadata(segment_path) {
                if let Ok(modified) = meta.modified() {
                    if let Ok(age) = std::time::SystemTime::now().duration_since(modified) {
                        if age.as_millis() > 1000 {
                            log::info!("Segment {} inactive for >1s. Proceeding.", fname);
                            return;
                        }
                    }
                }
            }

            log::debug!("Waiting for segment {} to finish...", fname);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        log::warn!("Timed out waiting for segment {} completion. Proceeding anyway.", fname);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_find_segments_by_time() {
        let temp_dir = std::env::temp_dir().join("squad_sync_test_time");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create segments
        // Segment 1: 2024-01-01 10:00:00 (Duration 2s) -> Ends 10:00:02
        // Segment 2: 2024-01-01 10:00:02 (Duration 2s) -> Ends 10:00:04
        // Segment 3: 2024-01-01 10:00:04 (Duration 2s) -> Ends 10:00:06

        let create_file = |name: &str| {
            File::create(temp_dir.join(name)).unwrap();
        };

        create_file("video_20240101100000.mkv");
        create_file("video_20240101100002.mkv");
        create_file("video_20240101100004.mkv");

        // Request: 10:00:01 to 10:00:03
        // Should include Segment 1 (ends 10:00:02 > 10:00:01) and Segment 2 (starts 10:00:02 < 10:00:03)
        // Segment 3 starts 10:00:04, which is > 10:00:03, so excluded.

        let start_naive = chrono::NaiveDateTime::parse_from_str("20240101100001", "%Y%m%d%H%M%S").unwrap();
        let end_naive = chrono::NaiveDateTime::parse_from_str("20240101100003", "%Y%m%d%H%M%S").unwrap();
        
        let start = Local.from_local_datetime(&start_naive).unwrap();
        let end = Local.from_local_datetime(&end_naive).unwrap();

        let segments = find_segments_by_time(&temp_dir, "video_", start, end).unwrap();
        
        assert_eq!(segments.len(), 2);
        assert!(segments[0].to_string_lossy().contains("20240101100000"));
        assert!(segments[1].to_string_lossy().contains("20240101100002"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
