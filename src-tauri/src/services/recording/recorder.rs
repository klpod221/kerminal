use crate::error::AppError;
use crate::models::recording::*;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

pub struct SessionRecorder {
    pub recording_id: String,
    pub session_name: String,
    pub terminal_type: String,
    start_time: Instant,
    pub started_at: DateTime<Utc>,
    pub file_path: String,
    file_writer: Arc<Mutex<File>>,
    pub header: AsciicastHeader,
    is_recording: Arc<tokio::sync::RwLock<bool>>,
}

impl SessionRecorder {
    pub async fn new(
        recording_id: String,
        file_path: String,
        width: u16,
        height: u16,
        title: Option<String>,
        terminal_type: String,
    ) -> Result<Self, AppError> {
        let file = File::create(&file_path)
            .await
            .map_err(|e| AppError::General(format!("Failed to create recording file: {}", e)))?;

        let header = AsciicastHeader {
            version: 2,
            width,
            height,
            timestamp: Some(chrono::Utc::now().timestamp()),
            title: title.clone(),
            env: Some(serde_json::json!({
                "TERM": "xterm-256color",
                "SHELL": std::env::var("SHELL").unwrap_or_default()
            })),
        };

        Ok(Self {
            recording_id,
            session_name: title.unwrap_or_else(|| "Unnamed Session".to_string()),
            terminal_type,
            start_time: Instant::now(),
            started_at: Utc::now(),
            file_path,
            file_writer: Arc::new(Mutex::new(file)),
            header,
            is_recording: Arc::new(tokio::sync::RwLock::new(true)),
        })
    }

    pub async fn write_header(&self) -> Result<(), AppError> {
        let mut writer = self.file_writer.lock().await;
        let header_json = serde_json::to_string(&self.header)
            .map_err(|e| AppError::serialization_error(e.to_string()))?;
        writer
            .write_all(header_json.as_bytes())
            .await
            .map_err(|e| AppError::General(format!("IO error: {}", e)))?;
        writer
            .write_all(b"\n")
            .await
            .map_err(|e| AppError::General(format!("IO error: {}", e)))?;
        Ok(())
    }

    pub async fn record_output(&self, data: &[u8]) -> Result<(), AppError> {
        if !*self.is_recording.read().await {
            return Ok(());
        }

        let elapsed = self.start_time.elapsed().as_secs_f64();
        let data_str = String::from_utf8_lossy(data);

        let event = serde_json::json!([elapsed, "o", data_str]);
        let event_line = format!(
            "{}\n",
            serde_json::to_string(&event)
                .map_err(|e| AppError::serialization_error(e.to_string()))?
        );

        let mut writer = self.file_writer.lock().await;
        writer
            .write_all(event_line.as_bytes())
            .await
            .map_err(|e| AppError::General(format!("IO error: {}", e)))?;

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), AppError> {
        *self.is_recording.write().await = false;
        let mut writer = self.file_writer.lock().await;
        writer
            .flush()
            .await
            .map_err(|e| AppError::General(format!("IO error: {}", e)))?;
        Ok(())
    }

    pub fn get_duration_ms(&self) -> i64 {
        self.start_time.elapsed().as_millis() as i64
    }
}
