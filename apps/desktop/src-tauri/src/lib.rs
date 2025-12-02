use tauri::Manager;
use std::os::windows::process::CommandExt;

pub mod ffmpeg;
pub mod commands;
pub mod state;
pub mod config;
pub mod audio;
pub mod error;
pub mod constants;
pub mod ntp;
#[cfg(target_os = "windows")]
pub mod job_object;

use state::RecordingState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_http::init())
    .plugin(tauri_plugin_log::Builder::default()
        .targets([
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir { file_name: Some("squad_sync".to_string()) }),
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
        ])
        .build())
    .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(|app, shortcut, event| {
        if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed
            && shortcut.matches(tauri_plugin_global_shortcut::Modifiers::ALT, tauri_plugin_global_shortcut::Code::F10) {
                log::info!("Global Hotkey Triggered");
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    match crate::commands::replay::save_replay_impl(&app_handle, None).await {
                        Ok(path) => log::info!("Replay saved via hotkey: {}", path),
                        Err(e) => log::error!("Failed to save replay via hotkey: {}", e),
                    }
                });
        }
    }).build())
    .manage(RecordingState::new())
    .invoke_handler(tauri::generate_handler![
        commands::recording::enable_replay,
        commands::recording::disable_replay,
        commands::replay::save_replay,
        commands::system::get_system_info,
        commands::config::get_config,
        commands::config::update_config,
        commands::devices::get_audio_devices,
        commands::devices::get_system_audio_devices,
        commands::devices::get_system_audio_devices,
        commands::monitors::get_monitors,
        commands::playback::get_recordings,
        commands::playback::delete_recording,
        commands::playback::rename_recording,
        commands::playback::show_in_folder,
        commands::playback::open_file,
        commands::playback::generate_thumbnail
    ])
    .setup(|app| {
      #[cfg(debug_assertions)]
      {
        if let Some(window) = app.get_webview_window("main") {
            window.open_devtools();
        }
      }
      
      // Load config
      let config = crate::config::AppConfig::load(app.handle());
      let state = app.state::<RecordingState>();
      match state.config.lock() {
          Ok(mut c) => *c = config.clone(),
          Err(e) => log::error!("Failed to lock config mutex: {}", e),
      }

      // Start NTP Sync
      state.ntp_manager.start();

      // Cleanup Temp Buffer on Startup
      let temp_path_str = config.recording.temp_path.replace("%TEMP%", &std::env::temp_dir().to_string_lossy());
      let buffer_dir = std::path::PathBuf::from(temp_path_str);
      if buffer_dir.exists() {
          log::info!("Cleaning up buffer directory: {:?}", buffer_dir);
          let _ = std::fs::remove_dir_all(&buffer_dir);
          let _ = std::fs::create_dir_all(&buffer_dir);
      }

      // Set High Priority for the main process
      let pid = std::process::id();
      std::thread::spawn(move || {
          const CREATE_NO_WINDOW: u32 = 0x08000000;
          let _ = std::process::Command::new("powershell")
              .args([
                  "-NoProfile", 
                  "-Command", 
                  &format!("Get-Process -Id {} | ForEach-Object {{ $_.PriorityClass = 'High' }}", pid)
              ])
              .creation_flags(CREATE_NO_WINDOW)
              .output();
          log::info!("Set Main Process (PID: {}) to High Priority", pid);
      });

      // Register Global Shortcut
      use tauri_plugin_global_shortcut::GlobalShortcutExt;
      if let Err(e) = app.handle().global_shortcut().register("Alt+F10") {
          log::error!("Failed to register global shortcut: {}", e);
      }



      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
