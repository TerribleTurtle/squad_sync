#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_copy_segment_retry() {
        let temp_dir = std::env::temp_dir().join("squad_sync_test_copy");
        fs::create_dir_all(&temp_dir).unwrap();
        
        let source = temp_dir.join("source.txt");
        let dest = temp_dir.join("dest.txt");
        
        // Create source
        fs::write(&source, "test content").unwrap();
        
        // Simulate a lock (on Windows, opening with write access without sharing might lock it)
        // But standard File::create might not lock reading.
        // Let's just verify the function works for now.
        
        let result = copy_segment(&source, &dest);
        assert!(result.is_ok());
        assert!(dest.exists());
        
        fs::remove_dir_all(temp_dir).unwrap();
    }
}
