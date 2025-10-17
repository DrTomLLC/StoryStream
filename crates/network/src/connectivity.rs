// crates/network/src/connectivity.rs
//! Network connectivity checks

use crate::client::Client;
use crate::error::{NetworkError, NetworkResult};

/// Network connectivity checker
#[derive(Clone)]
pub struct ConnectivityChecker {
    client: Client,
    check_urls: Vec<String>,
}

impl ConnectivityChecker {
    /// Creates a new connectivity checker with default URLs
    pub fn new(client: Client) -> Self {
        Self {
            client,
            check_urls: vec![
                "https://www.google.com".to_string(),
                "https://www.cloudflare.com".to_string(),
                "https://www.rust-lang.org".to_string(),
            ],
        }
    }

    /// Creates a connectivity checker with custom URLs
    pub fn with_urls(client: Client, urls: Vec<String>) -> Self {
        Self {
            client,
            check_urls: urls,
        }
    }

    /// Checks if network is available
    pub async fn is_online(&self) -> bool {
        for url in &self.check_urls {
            if self.client.is_accessible(url).await {
                return true;
            }
        }
        false
    }

    /// Checks network connectivity and returns error if offline
    pub async fn check(&self) -> NetworkResult<()> {
        if self.is_online().await {
            Ok(())
        } else {
            Err(NetworkError::NetworkUnavailable)
        }
    }

    /// Estimates network latency by timing a HEAD request
    pub async fn estimate_latency(&self, url: &str) -> NetworkResult<std::time::Duration> {
        let start = std::time::Instant::now();
        self.client.head(url).await?;
        Ok(start.elapsed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connectivity_checker_creation() {
        let client = Client::new().expect("Failed to create client");
        let _checker = ConnectivityChecker::new(client);
    }

    #[test]
    fn test_connectivity_checker_with_custom_urls() {
        let client = Client::new().expect("Failed to create client");
        let urls = vec!["https://example.com".to_string()];
        let _checker = ConnectivityChecker::with_urls(client, urls);
    }

    #[tokio::test]
    async fn test_is_online() {
        let client = Client::new().expect("Failed to create client");
        let checker = ConnectivityChecker::new(client);

        // This test might fail in CI/offline environments
        let _ = checker.is_online().await;
    }
}