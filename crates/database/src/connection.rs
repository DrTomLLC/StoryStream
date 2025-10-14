//! Database connection management

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{Pool, Sqlite};
use std::path::Path;
use std::str::FromStr;
use storystream_core::AppError;

/// Database connection pool
pub type DbPool = Pool<Sqlite>;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Path to the SQLite database file
    pub path: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Enable Write-Ahead Logging (WAL) mode
    pub enable_wal: bool,
    /// Create database if it doesn't exist
    pub create_if_missing: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: "storystream.db".to_string(),
            max_connections: 10,
            enable_wal: true,
            create_if_missing: true,
        }
    }
}

impl DatabaseConfig {
    /// Creates a new configuration with a custom path
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }

    /// Sets the maximum number of connections
    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Enables or disables WAL mode
    pub fn with_wal(mut self, enable: bool) -> Self {
        self.enable_wal = enable;
        self
    }

    /// Sets whether to create the database if missing
    pub fn with_create_if_missing(mut self, create: bool) -> Self {
        self.create_if_missing = create;
        self
    }
}

/// Establishes a connection pool to the database
pub async fn connect(config: DatabaseConfig) -> Result<DbPool, AppError> {
    // Build connection options
    let mut options = SqliteConnectOptions::from_str(&format!("sqlite:{}", config.path))
        .map_err(|e| AppError::database("Invalid database path", e))?
        .create_if_missing(config.create_if_missing);

    // Configure WAL mode for better concurrency
    if config.enable_wal {
        options = options
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);
    }

    // Create connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(config.max_connections)
        .connect_with(options)
        .await
        .map_err(|e| AppError::database("Failed to connect to database", e))?;

    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await
        .map_err(|e| AppError::database("Failed to enable foreign keys", e))?;

    Ok(pool)
}

/// Creates an in-memory database for testing
#[cfg(test)]
pub async fn create_test_db() -> Result<DbPool, AppError> {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .map_err(|e| AppError::database("Failed to create test database", e))?
        .journal_mode(SqliteJournalMode::Memory);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|e| AppError::database("Failed to connect to test database", e))?;

    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await
        .map_err(|e| AppError::database("Failed to enable foreign keys", e))?;

    Ok(pool)
}

/// Closes the database connection pool
pub async fn close(pool: DbPool) {
    pool.close().await;
}

/// Checks if the database file exists
pub fn database_exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_connect_creates_database() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let config = DatabaseConfig::new(path.clone());
        let pool = connect(config).await.unwrap();

        assert!(database_exists(&path));
        close(pool).await;
    }

    #[tokio::test]
    async fn test_connect_with_wal_mode() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap().to_string();

        let config = DatabaseConfig::new(path).with_wal(true);
        let pool = connect(config).await.unwrap();

        // Verify WAL mode is enabled
        let result: (String,) = sqlx::query_as("PRAGMA journal_mode;")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result.0.to_lowercase(), "wal");
        close(pool).await;
    }

    #[tokio::test]
    async fn test_foreign_keys_enabled() {
        let pool = create_test_db().await.unwrap();

        let result: (i32,) = sqlx::query_as("PRAGMA foreign_keys;")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result.0, 1); // Foreign keys enabled
        close(pool).await;
    }

    #[tokio::test]
    async fn test_config_builder() {
        let config = DatabaseConfig::new("test.db")
            .with_max_connections(20)
            .with_wal(false)
            .with_create_if_missing(false);

        assert_eq!(config.path, "test.db");
        assert_eq!(config.max_connections, 20);
        assert!(!config.enable_wal);
        assert!(!config.create_if_missing);
    }

    #[tokio::test]
    async fn test_create_test_db() {
        let pool = create_test_db().await.unwrap();

        // Verify we can execute queries
        sqlx::query("SELECT 1;")
            .execute(&pool)
            .await
            .unwrap();

        close(pool).await;
    }

    #[tokio::test]
    async fn test_database_exists() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        assert!(database_exists(path));
        assert!(!database_exists("/nonexistent/path/to/db.sqlite"));
    }

    #[tokio::test]
    async fn test_config_default() {
        let config = DatabaseConfig::default();

        assert_eq!(config.path, "storystream.db");
        assert_eq!(config.max_connections, 10);
        assert!(config.enable_wal);
        assert!(config.create_if_missing);
    }
}