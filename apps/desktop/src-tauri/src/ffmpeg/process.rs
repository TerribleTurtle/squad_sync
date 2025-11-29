use tauri::{AppHandle, Manager};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandEvent, CommandChild};
use crate::ffmpeg::commands::FfmpegCommandBuilder;

pub fn spawn_ffmpeg(app: &AppHandle) -> Result<CommandChild, String> {
    let app_cache = app.path().app_cache_dir().map_err(|e| e.to_string())?;
    let buffer_dir = app_cache.join("buffer");
    
    // Ensure buffer directory exists
    if !buffer_dir.exists() {
        std::fs::create_dir_all(&buffer_dir).map_err(|e| e.to_string())?;
    }

    let output_pattern = buffer_dir.join("out_%03d.ts");
    let output_path_str = output_pattern.to_string_lossy().to_string();

    let builder = FfmpegCommandBuilder::new(output_path_str);
    let args = builder.build();

    println!("Spawning FFmpeg with args: {:?}", args);

    let sidecar_command = app.shell().sidecar("ffmpeg").map_err(|e| e.to_string())?;
    let (mut rx, child) = sidecar_command
        .args(args)
        .spawn()
        .map_err(|e| e.to_string())?;

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line) => {
                    println!("FFmpeg: {:?}", String::from_utf8(line));
                }
                CommandEvent::Stderr(line) => {
                    eprintln!("FFmpeg Error: {:?}", String::from_utf8(line));
                }
                _ => {}
            }
        }
    });
    
    Ok(child)
}
