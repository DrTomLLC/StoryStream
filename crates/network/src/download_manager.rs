// crates/network/src/download_manager.rs
//! Advanced download manager with queue, resume, and concurrency control

use crate::client::Client;
use crate::error::{NetworkError, NetworkResult};
use crate::progress::ProgressTracker;
use bytes::Bytes;
use futures::StreamExt;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
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

struct DownloadManagerState {
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
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {
                    let task_opt = {
                        let mut state_guard = state.write().await;
                        state_guard.queue.pop_front()
                    };

                    if let Some(task) = task_opt {
                        let permit = match semaphore.clone().try_acquire_owned() {
                            Ok(p) => p,
                            Err(_) => {
                                let mut state_guard = state.write().await;
                                state_guard.queue.push_front(task);
                                continue;
                            }
                        };

                        let task_id = task.id.clone();
                        let client_clone = client.clone();

                        {
                            let mut state_guard = state.write().await;
                            state_guard.status.insert(task_id.clone(), DownloadStatus::InProgress);
                        }

                        let handle = tokio::spawn(async move {
                            let result = Self::execute_download(client_clone, task).await;
                            drop(permit);
                            result
                        });

                        {
                            let mut state_guard = state.write().await;
                            state_guard.active.insert(task_id, handle);
                        }
                    }
                }
            }
        }
    }

    async fn execute_download(client: Client, task: DownloadTask) -> NetworkResult<u64> {
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

        state.status.insert(id.to_string(), DownloadStatus::Cancelled);
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