// crates/network/src/resume.rs
//! Download resume capability

use crate::error::{NetworkError, NetworkResult};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeInfo {
    pub bytes_downloaded: u64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub total_size: Option<u64>,
    pub interrupted_at: chrono::DateTime<chrono::Utc>,
}

impl ResumeInfo {
    pub fn new(bytes_downloaded: u64) -> Self {
        Self {
            bytes_downloaded,
            etag: None,
            last_modified: None,
            total_size: None,
            interrupted_at: chrono::Utc::now(),
        }
    }

    pub fn with_etag(mut self, etag: String) -> Self {
        self.etag = Some(etag);
        self
    }

    pub fn with_last_modified(mut self, last_modified: String) -> Self {
        self.last_modified = Some(last_modified);
        self
    }

    pub fn with_total_size(mut self, total_size: u64) -> Self {
        self.total_size = Some(total_size);
        self
    }

    pub fn is_valid(&self) -> bool {
        self.bytes_downloaded > 0
    }

    pub fn is_complete(&self) -> bool {
        if let Some(total) = self.total_size {
            self.bytes_downloaded >= total
        } else {
            false
        }
    }

    pub fn progress_percentage(&self) -> Option<f64> {
        self.total_size.map(|total| {
            if total > 0 {
                (self.bytes_downloaded as f64 / total as f64) * 100.0
            } else {
                0.0
            }
        })
    }
}

pub struct ResumeManager {
    metadata_dir: std::path::PathBuf,
}

impl ResumeManager {
    pub fn new(metadata_dir: impl AsRef<Path>) -> NetworkResult<Self> {
        let metadata_dir = metadata_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&metadata_dir).map_err(NetworkError::Io)?;

        Ok(Self { metadata_dir })
    }

    pub async fn save(&self, download_id: &str, info: &ResumeInfo) -> NetworkResult<()> {
        let path = self.metadata_path(download_id);
        let json =
            serde_json::to_string_pretty(info).map_err(|e| NetworkError::Custom(e.to_string()))?;
        fs::write(&path, json).await?;
        Ok(())
    }

    pub async fn load(&self, download_id: &str) -> NetworkResult<Option<ResumeInfo>> {
        let path = self.metadata_path(download_id);

        if !path.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&path).await?;
        let info: ResumeInfo =
            serde_json::from_str(&json).map_err(|e| NetworkError::Custom(e.to_string()))?;

        Ok(Some(info))
    }

    pub async fn delete(&self, download_id: &str) -> NetworkResult<()> {
        let path = self.metadata_path(download_id);
        if path.exists() {
            fs::remove_file(&path).await?;
        }
        Ok(())
    }

    pub fn has_resume_info(&self, download_id: &str) -> bool {
        self.metadata_path(download_id).exists()
    }

    fn metadata_path(&self, download_id: &str) -> std::path::PathBuf {
        self.metadata_dir.join(format!("{}.json", download_id))
    }

    pub async fn list_incomplete(&self) -> NetworkResult<Vec<(String, ResumeInfo)>> {
        let mut incomplete = Vec::new();

        let mut entries = fs::read_dir(&self.metadata_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(stem) = path.file_stem() {
                    let download_id = stem.to_string_lossy().to_string();
                    if let Some(info) = self.load(&download_id).await? {
                        if !info.is_complete() {
                            incomplete.push((download_id, info));
                        }
                    }
                }
            }
        }

        Ok(incomplete)
    }

    pub async fn cleanup_old(&self, days: u64) -> NetworkResult<usize> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let mut removed = 0;

        let mut entries = fs::read_dir(&self.metadata_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(stem) = path.file_stem() {
                    let download_id = stem.to_string_lossy().to_string();
                    if let Some(info) = self.load(&download_id).await? {
                        if info.interrupted_at < cutoff {
                            self.delete(&download_id).await?;
                            removed += 1;
                        }
                    }
                }
            }
        }

        Ok(removed)
    }
}

pub async fn can_resume(
    file_path: impl AsRef<Path>,
    resume_info: &ResumeInfo,
) -> NetworkResult<bool> {
    let file_path = file_path.as_ref();

    if !file_path.exists() {
        return Ok(false);
    }

    let metadata = fs::metadata(file_path).await?;
    if metadata.len() != resume_info.bytes_downloaded {
        return Ok(false);
    }

    if let Some(total) = resume_info.total_size {
        if metadata.len() > total {
            return Ok(false);
        }
    }

    Ok(true)
}