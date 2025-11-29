use tauri::Manager;

pub mod ffmpeg;
pub mod commands;
pub mod state;
pub mod config;
pub mod audio;
pub mod error;

use state::RecordingState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_http::init())
    .manage(RecordingState::new())
    .invoke_handler(tauri::generate_handler![
        commands::recording::start_recording,
        commands::recording::stop_recording,
        commands::clip::create_clip,
        commands::upload::upload_clip,
        commands::system::get_system_info,
        commands::config::get_config,
        commands::config::update_config,
        commands::devices::get_audio_devices
    ])
    .setup(|app| {
      #[cfg(debug_assertions)]
      {
        let window = app.get_webview_window("main").unwrap();
        window.open_devtools();
      }
      
      // Load config
      let config = crate::config::AppConfig::load(app.handle());
      let state = app.state::<RecordingState>();
      *state.config.lock().unwrap() = config;

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
