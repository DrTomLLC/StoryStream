// crates/network/src/client.rs
//! HTTP client wrapper with resilience

use crate::error::{NetworkError, NetworkResult};
use reqwest::{Client as ReqwestClient, Response};
use std::time::Duration;
use storystream_resilience::{CircuitBreaker, CircuitBreakerConfig, RetryPolicy};

/// HTTP client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Request timeout
    pub timeout: Duration,
    /// User agent string
    pub user_agent: String,
    /// Maximum redirects to follow
    pub max_redirects: usize,
    /// Retry policy
    pub retry_policy: Option<RetryPolicy>,
    /// Circuit breaker config
    pub circuit_breaker_config: Option<CircuitBreakerConfig>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            user_agent: format!("StoryStream/{}", env!("CARGO_PKG_VERSION")),
            max_redirects: 10,
            retry_policy: Some(RetryPolicy::new(3).with_initial_delay(Duration::from_millis(100))),
            circuit_breaker_config: Some(CircuitBreakerConfig::new(5, Duration::from_secs(60))),
        }
    }
}

/// HTTP client with resilience features
#[derive(Clone)]
pub struct Client {
    inner: ReqwestClient,
    config: ClientConfig,
    circuit_breaker: Option<CircuitBreaker>,
}

impl Client {
    /// Creates a new client with default configuration
    pub fn new() -> NetworkResult<Self> {
        Self::with_config(ClientConfig::default())
    }

    /// Creates a new client with custom configuration
    pub fn with_config(config: ClientConfig) -> NetworkResult<Self> {
        let client = ReqwestClient::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent)
            .redirect(reqwest::redirect::Policy::limited(config.max_redirects))
            .build()
            .map_err(NetworkError::Http)?;

        let circuit_breaker = config
            .circuit_breaker_config
            .as_ref()
            .map(|cfg| CircuitBreaker::new(cfg.clone()));

        Ok(Self {
            inner: client,
            config,
            circuit_breaker,
        })
    }

    /// Performs a GET request
    pub async fn get(&self, url: &str) -> NetworkResult<Response> {
        self.request(|| async { self.inner.get(url).send().await }).await
    }

    /// Performs a HEAD request
    pub async fn head(&self, url: &str) -> NetworkResult<Response> {
        self.request(|| async { self.inner.head(url).send().await }).await
    }

    /// Gets the content length of a URL without downloading
    pub async fn content_length(&self, url: &str) -> NetworkResult<Option<u64>> {
        let response = self.head(url).await?;
        Ok(response.content_length())
    }

    /// Checks if a URL is accessible
    pub async fn is_accessible(&self, url: &str) -> bool {
        self.head(url).await.is_ok()
    }

    /// Internal request handler with resilience
    async fn request<F, Fut>(&self, request_fn: F) -> NetworkResult<Response>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<Response, reqwest::Error>>,
    {
        // Check circuit breaker
        if let Some(cb) = &self.circuit_breaker {
            cb.can_proceed()?;
        }

        // Execute request with retry
        let mut attempts = 0;
        let max_attempts = self.config.retry_policy.as_ref().map(|p| p.max_attempts()).unwrap_or(1);

        loop {
            attempts += 1;

            match request_fn().await {
                Ok(response) => {
                    // Record success in circuit breaker
                    if let Some(cb) = &self.circuit_breaker {
                        cb.record_success();
                    }

                    // Check for HTTP errors
                    if response.status().is_success() {
                        return Ok(response);
                    } else {
                        let status = response.status();
                        let error = NetworkError::Custom(format!("HTTP {}: {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")));

                        // Don't retry client errors (4xx)
                        if status.is_client_error() {
                            if let Some(cb) = &self.circuit_breaker {
                                cb.record_failure();
                            }
                            return Err(error);
                        }

                        // Retry server errors (5xx) if we have attempts left
                        if attempts < max_attempts {
                            if let Some(cb) = &self.circuit_breaker {
                                cb.record_failure();
                            }

                            if let Some(policy) = &self.config.retry_policy {
                                let delay = policy.delay_for_attempt(attempts);
                                tokio::time::sleep(delay).await;
                            }
                            continue;
                        }

                        return Err(error);
                    }
                }
                Err(e) => {
                    // Record failure in circuit breaker
                    if let Some(cb) = &self.circuit_breaker {
                        cb.record_failure();
                    }

                    // Retry if we have attempts left
                    if attempts < max_attempts {
                        if let Some(policy) = &self.config.retry_policy {
                            let delay = policy.delay_for_attempt(attempts);
                            tokio::time::sleep(delay).await;
                        }
                        continue;
                    }

                    return Err(NetworkError::Http(e));
                }
            }
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new().expect("Failed to create default client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_redirects, 10);
        assert!(config.retry_policy.is_some());
    }

    #[test]
    fn test_client_creation() {
        let client = Client::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_custom_config() {
        let config = ClientConfig {
            timeout: Duration::from_secs(10),
            user_agent: "TestAgent".to_string(),
            max_redirects: 5,
            retry_policy: None,
            circuit_breaker_config: None,
        };

        let client = Client::with_config(config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_client_head_request() {
        let client = Client::new().expect("Failed to create client");

        // Test with a reliable public URL
        let result = client.head("https://www.rust-lang.org").await;

        // This might fail in CI/offline environments, so we just check it doesn't panic
        let _ = result;
    }
}