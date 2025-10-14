//! Database migrations

use crate::DbPool;
use storystream_core::AppError;

/// Migration 001: Initial schema
const MIGRATION_001: &str = include_str!("../migrations/001_initial_schema.sql");

/// Migration 002: Playlists placeholder
const MIGRATION_002: &str = include_str!("../migrations/002_playlists.sql");

/// Migration 003: Full-text search
const MIGRATION_003: &str = include_str!("../migrations/003_full_text_search.sql");

/// Migration 004: Add indexes
const MIGRATION_004: &str = include_str!("../migrations/004_add_indexes.sql");

/// Migration 005: Populate FTS tables
const MIGRATION_005: &str = include_str!("../migrations/005_populate_fts.sql");

/// Current database schema version
pub const CURRENT_VERSION: i64 = 5;

/// Returns the current migration version
pub fn current_version() -> i64 {
    CURRENT_VERSION
}

/// Runs all pending migrations
pub async fn run_migrations(pool: &DbPool) -> Result<(), AppError> {
    // Create migrations table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now') * 1000)
        )
        "#,
    )
        .execute(pool)
        .await
        .map_err(|e| AppError::database("Failed to create migrations table", e))?;

    // Run each migration
    run_migration(pool, 1, MIGRATION_001).await?;
    run_migration(pool, 2, MIGRATION_002).await?;
    run_migration(pool, 3, MIGRATION_003).await?;
    run_migration(pool, 4, MIGRATION_004).await?;
    run_migration(pool, 5, MIGRATION_005).await?;

    Ok(())
}

/// Runs a single migration if not already applied
async fn run_migration(pool: &DbPool, version: i64, sql: &str) -> Result<(), AppError> {
    // Check if migration already applied
    let applied: Option<i64> = sqlx::query_scalar("SELECT version FROM schema_migrations WHERE version = ?")
        .bind(version)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::database("Failed to check migration status", e))?;

    if applied.is_some() {
        return Ok(());
    }

    // Execute migration
    sqlx::query(sql)
        .execute(pool)
        .await
        .map_err(|e| AppError::database(&format!("Failed to run migration {}", version), e))?;

    Ok(())
}

/// Verifies database integrity
pub async fn verify_integrity(pool: &DbPool) -> Result<(), AppError> {
    let result: String = sqlx::query_scalar("PRAGMA integrity_check")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::database("Failed to check integrity", e))?;

    if result != "ok" {
        return Err(AppError::database(
            &format!("Database integrity check failed: {}", result),
            std::io::Error::new(std::io::ErrorKind::Other, "Integrity check failed"),
        ));
    }

    Ok(())
}

/// Optimizes the database
pub async fn optimize(pool: &DbPool) -> Result<(), AppError> {
    sqlx::query("PRAGMA optimize")
        .execute(pool)
        .await
        .map_err(|e| AppError::database("Failed to optimize database", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::create_test_db;

    #[tokio::test]
    async fn test_run_migrations() {
        let pool = create_test_db().await.unwrap();
        run_migrations(&pool).await.unwrap();

        // Verify all migrations were applied
        let versions: Vec<i64> = sqlx::query_scalar("SELECT version FROM schema_migrations ORDER BY version")
            .fetch_all(&pool)
            .await
            .unwrap();

        assert_eq!(versions, vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_verify_integrity() {
        let pool = create_test_db().await.unwrap();
        run_migrations(&pool).await.unwrap();

        verify_integrity(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn test_optimize() {
        let pool = create_test_db().await.unwrap();
        run_migrations(&pool).await.unwrap();

        optimize(&pool).await.unwrap();
    }
}