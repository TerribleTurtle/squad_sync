use tauri::command;
use std::path::PathBuf;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

#[command]
pub async fn upload_clip_to_url(file_path: String, upload_url: String) -> Result<(), String> {
    log::info!("Starting upload for {} to {}", file_path, upload_url);

    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    let file = File::open(&path).await.map_err(|e| format!("Failed to open file: {}", e))?;
    let metadata = file.metadata().await.map_err(|e| format!("Failed to get file metadata: {}", e))?;
    let file_size = metadata.len();
    let stream = ReaderStream::new(file);
    let body = reqwest::Body::wrap_stream(stream);

    let client = reqwest::Client::new();
    let response = client
        .put(&upload_url)
        .header("Content-Type", "video/mp4")
        .header("Content-Length", file_size)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Upload request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Upload failed with status: {}", response.status()));
    }

    log::info!("Upload successful for {}", file_path);
    Ok(())
}

#[command]
pub fn upload_clip() {
    log::info!("Upload clip command received (Legacy)");
}
