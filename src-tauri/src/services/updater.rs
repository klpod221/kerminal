use semver::Version;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_updater::UpdaterExt;
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize)]
pub struct LinuxUpdateInfo {
    pub available: bool,
    pub version: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct TauriUpdateInfo {
    version: String,
    date: Option<String>,
    body: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LatestJson {
    version: String,
    notes: Option<String>,
    pub_date: Option<String>,
    platforms: serde_json::Value, // We just need version for now
}

pub struct UpdaterService<R: Runtime> {
    app_handle: AppHandle<R>,
}

impl<R: Runtime> UpdaterService<R> {
    pub fn new(app_handle: AppHandle<R>) -> Self {
        Self { app_handle }
    }

    pub fn start_update_check_loop(&self) {
        let app_handle = self.app_handle.clone();

        tauri::async_runtime::spawn(async move {
            // Initial delay
            sleep(Duration::from_secs(10)).await;

            loop {
                if let Err(e) = Self::check_and_notify(&app_handle).await {
                    eprintln!("Failed to check for updates: {}", e);
                }

                sleep(Duration::from_secs(86400)).await;
            }
        });
    }

    async fn check_and_notify(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
        if cfg!(target_os = "linux") {
            Self::check_linux_update(app).await?;
        } else {
            if let Some(update) = app.updater()?.check().await? {
                let info = TauriUpdateInfo {
                    version: update.version.clone(),
                    date: update.date.map(|d| d.to_string()),
                    body: update.body.clone(),
                };
                app.emit("update-available", info)?;
            }
        }
        Ok(())
    }

    async fn check_linux_update(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        let url = "https://github.com/klpod221/kerminal/releases/latest/download/latest.json";

        let res = client
            .get(url)
            .header("User-Agent", "kerminal")
            .send()
            .await?;

        if res.status().is_success() {
            let text = res.text().await?;

            let latest_info: LatestJson = serde_json::from_str(&text)?;
            let latest_version = latest_info.version;

            let current_version = app.package_info().version.to_string();

            let clean_latest = latest_version.trim_start_matches('v');
            let clean_current = current_version.trim_start_matches('v');

            // Use semver comparison instead of string comparison
            let is_newer = match (Version::parse(clean_latest), Version::parse(clean_current)) {
                (Ok(latest), Ok(current)) => latest > current,
                _ => clean_latest != clean_current, // Fallback to string comparison if parsing fails
            };

            if is_newer {
                let info = LinuxUpdateInfo {
                    available: true,
                    version: Some(format!("v{}", clean_latest)),
                    url: Some(format!(
                        "https://github.com/klpod221/kerminal/releases/tag/v{}",
                        clean_latest
                    )),
                };
                app.emit("update-available", info)?;
            }
        }
        Ok(())
    }
}
