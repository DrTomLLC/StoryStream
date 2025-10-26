// crates/network/src/download_manager.rs
//! Advanced download manager with queue, resume, and concurrency control

use crate::client::Client;
use crate::error::{NetworkError, NetworkResult};
use futures::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, Mutex, RwLock, Semaphore};
use tokio::task::JoinHandle;

/// Download priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Download status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadStatus {
    Queued,
    InProgress,
    Paused,
    Completed,
    Failed(String),
    Cancelled,
}

/// Progress callback type
pub type ProgressCallback = Arc<dyn Fn(u64, Option<u64>) + Send + Sync>;

/// Download task configuration
#[derive(Clone)]
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub destination: PathBuf,
    pub priority: Priority,
    pub resume_allowed: bool,
    pub progress_callback: Option<ProgressCallback>,
}

impl DownloadTask {
    pub fn new(id: String, url: String, destination: PathBuf) -> Self {
        Self {
            id,
            url,
            destination,
            priority: Priority::Normal,
            resume_allowed: true,
            progress_callback: None,
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_progress_callback(mut self, callback: ProgressCallback) -> Self {
        self.progress_callback = Some(callback);
        self
    }

    pub fn with_resume(mut self, allowed: bool) -> Self {
        self.resume_allowed = allowed;
        self
    }
}

/// Download manager configuration
#[derive(Debug, Clone)]
pub struct DownloadManagerConfig {
    pub max_concurrent: usize,
    pub auto_resume: bool,
    pub max_resume_attempts: usize,
    pub bandwidth_limit: Option<u64>,
    pub chunk_size: usize,
    pub verify_integrity: bool,
}

impl Default for DownloadManagerConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 3,
            auto_resume: true,
            max_resume_attempts: 5,
            bandwidth_limit: None,
            chunk_size: 8192,
            verify_integrity: false,
        }
    }
}

// Made public to fix visibility warning
pub struct DownloadManagerState {
    queue: VecDeque<DownloadTask>,
    active: HashMap<String, JoinHandle<NetworkResult<u64>>>,
    status: HashMap<String, DownloadStatus>,
}

pub struct AdvancedDownloadManager {
    client: Client,
    config: DownloadManagerConfig,
    pub state: Arc<RwLock<DownloadManagerState>>,
    semaphore: Arc<Semaphore>,
    shutdown_tx: mpsc::Sender<()>,
    shutdown_rx: Arc<Mutex<mpsc::Receiver<()>>>,
}

impl AdvancedDownloadManager {
    pub fn new(client: Client, config: DownloadManagerConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent));

        let state = Arc::new(RwLock::new(DownloadManagerState {
            queue: VecDeque::new(),
            active: HashMap::new(),
            status: HashMap::new(),
        }));

        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);

        Self {
            client,
            config,
            state,
            semaphore,
            shutdown_tx,
            shutdown_rx: Arc::new(Mutex::new(shutdown_rx)),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &DownloadManagerConfig {
        &self.config
    }

    pub async fn enqueue(&self, task: DownloadTask) -> NetworkResult<()> {
        let mut state = self.state.write().await;

        if state.status.contains_key(&task.id) {
            return Err(NetworkError::Custom(format!(
                "Download {} already exists",
                task.id
            )));
        }

        let insert_pos = state
            .queue
            .iter()
            .position(|t| t.priority < task.priority)
            .unwrap_or(state.queue.len());

        state.queue.insert(insert_pos, task.clone());
        state.status.insert(task.id.clone(), DownloadStatus::Queued);

        Ok(())
    }

    pub async fn start(&self) {
        let state = Arc::clone(&self.state);
        let client = self.client.clone();
        let semaphore = Arc::clone(&self.semaphore);
        let mut shutdown_rx = self.shutdown_rx.lock().await;

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    log::info!("Download manager shutting down");
                    break;
                }
                _ = async {
                    let task = {
                        let mut state = state.write().await;
                        state.queue.pop_front()
                    };

                    if let Some(task) = task {
                        let permit = semaphore.clone().acquire_owned().await.ok();

                        if let Some(_permit) = permit {
                            let task_id = task.id.clone();
                            let client = client.clone();
                            let state = Arc::clone(&state);

                            let handle = tokio::spawn(async move {
                                let result = Self::download_task(&client, &task).await;
                                drop(_permit);
                                result
                            });

                            state.write().await.active.insert(task_id.clone(), handle);
                            state.write().await.status.insert(task_id, DownloadStatus::InProgress);
                        }
                    } else {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                } => {}
            }
        }
    }

    async fn download_task(client: &Client, task: &DownloadTask) -> NetworkResult<u64> {
        let response = client.get(&task.url).await?;
        let total_size = response.content_length();

        let mut file = File::create(&task.destination).await?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(NetworkError::Http)?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            if let Some(ref callback) = task.progress_callback {
                callback(downloaded, total_size);
            }
        }

        file.flush().await?;
        Ok(downloaded)
    }

    pub async fn cancel(&self, id: &str) -> NetworkResult<()> {
        let mut state = self.state.write().await;
        state.queue.retain(|t| t.id != id);

        if let Some(handle) = state.active.remove(id) {
            handle.abort();
        }

        state
            .status
            .insert(id.to_string(), DownloadStatus::Cancelled);
        Ok(())
    }

    pub async fn get_status(&self, id: &str) -> Option<DownloadStatus> {
        let state = self.state.read().await;
        state.status.get(id).cloned()
    }

    pub async fn active_count(&self) -> usize {
        let state = self.state.read().await;
        state.active.len()
    }

    pub async fn queue_length(&self) -> usize {
        let state = self.state.read().await;
        state.queue.len()
    }

    pub async fn shutdown(&self) -> NetworkResult<()> {
        self.shutdown_tx
            .send(())
            .await
            .map_err(|e| NetworkError::Custom(format!("Shutdown failed: {}", e)))?;

        let mut state = self.state.write().await;
        for (_, handle) in state.active.drain() {
            handle.abort();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download_manager_creation() {
        let client = Client::new().unwrap();
        let config = DownloadManagerConfig::default();
        let _manager = AdvancedDownloadManager::new(client, config);
    }

    #[tokio::test]
    async fn test_enqueue_task() {
        let client = Client::new().unwrap();
        let config = DownloadManagerConfig::default();
        let manager = AdvancedDownloadManager::new(client, config);

        let task = DownloadTask::new(
            "test".to_string(),
            "https://example.com/file".to_string(),
            PathBuf::from("/tmp/test"),
        );

        assert!(manager.enqueue(task).await.is_ok());
    }

    #[tokio::test]
    async fn test_duplicate_enqueue() {
        let client = Client::new().unwrap();
        let config = DownloadManagerConfig::default();
        let manager = AdvancedDownloadManager::new(client, config);

        let task1 = DownloadTask::new(
            "test".to_string(),
            "https://example.com/file".to_string(),
            PathBuf::from("/tmp/test"),
        );

        let task2 = task1.clone();

        assert!(manager.enqueue(task1).await.is_ok());
        assert!(manager.enqueue(task2).await.is_err());
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let client = Client::new().unwrap();
        let config = DownloadManagerConfig::default();
        let manager = AdvancedDownloadManager::new(client, config);

        let low = DownloadTask::new(
            "low".to_string(),
            "https://example.com/low".to_string(),
            PathBuf::from("/tmp/low"),
        )
            .with_priority(Priority::Low);

        let high = DownloadTask::new(
            "high".to_string(),
            "https://example.com/high".to_string(),
            PathBuf::from("/tmp/high"),
        )
            .with_priority(Priority::High);

        manager.enqueue(low).await.unwrap();
        manager.enqueue(high).await.unwrap();

        let state = manager.state.read().await;
        assert_eq!(state.queue.len(), 2);
        assert_eq!(state.queue[0].id, "high");
        assert_eq!(state.queue[1].id, "low");
    }

    #[tokio::test]
    async fn test_config_accessor() {
        let client = Client::new().unwrap();
        let config = DownloadManagerConfig {
            max_concurrent: 5,
            ..Default::default()
        };
        let manager = AdvancedDownloadManager::new(client, config);

        assert_eq!(manager.config().max_concurrent, 5);
    }
}