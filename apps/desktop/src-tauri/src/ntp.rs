use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use log::{info, error, warn};
use rsntp::SntpClient;

pub struct NtpManager {
    offset_ms: Arc<AtomicI64>,
}

impl NtpManager {
    pub fn new() -> Self {
        Self {
            offset_ms: Arc::new(AtomicI64::new(0)),
        }
    }

    pub fn start(&self) {
        let offset = self.offset_ms.clone();
        
        tauri::async_runtime::spawn(async move {
            let mut ticker = interval(Duration::from_secs(15 * 60)); // 15 minutes

            loop {
                ticker.tick().await;
                Self::sync(&offset).await;
            }
        });
    }

    async fn sync(offset_store: &Arc<AtomicI64>) {
        info!("Starting NTP Sync (Multi-Sample)...");
        
        let result = tauri::async_runtime::spawn_blocking(|| {
            let client = SntpClient::new();
            let mut best_offset = None;
            let mut min_delay_ms = f64::MAX;
            let samples = 5;
            let mut success_count = 0;

            for i in 1..=samples {
                match client.synchronize("pool.ntp.org") {
                    Ok(result) => {
                        let delay = result.round_trip_delay();
                        let offset = result.clock_offset();
                        
                        let delay_ms = delay.as_secs_f64() * 1000.0;
                        let offset_ms = offset.as_secs_f64() * 1000.0;
                        
                        info!("NTP Sample {}/{}: Offset={:.2}ms, Delay={:.2}ms", 
                            i, samples, offset_ms, delay_ms
                        );

                        if delay_ms < min_delay_ms {
                            min_delay_ms = delay_ms;
                            best_offset = Some(offset_ms as i64);
                        }
                        success_count += 1;
                    },
                    Err(e) => {
                        warn!("NTP Sample {}/{} Failed: {}", i, samples, e);
                    }
                }
                // Small delay between samples to avoid flooding
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            if success_count > 0 {
                best_offset.ok_or("Failed to get any valid offset".to_string())
            } else {
                Err("All NTP samples failed".to_string())
            }
        }).await;

        match result {
            Ok(Ok(offset)) => {
                offset_store.store(offset, Ordering::Relaxed);
                info!("NTP Sync Successful. Best Offset: {} ms", offset);
            },
            Ok(Err(e)) => {
                warn!("NTP Sync Failed: {}. Using previous offset.", e);
            },
            Err(e) => {
                error!("NTP Task Panicked: {}", e);
            }
        }
    }

    pub fn get_offset(&self) -> i64 {
        self.offset_ms.load(Ordering::Relaxed)
    }

    pub fn get_ntp_time_ms(&self) -> u64 {
        let system_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let offset = self.get_offset();
        
        // Apply offset safely
        if offset >= 0 {
            system_time + (offset as u64)
        } else {
            system_time.saturating_sub((-offset) as u64)
        }
    }
}
