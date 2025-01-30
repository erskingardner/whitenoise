use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{path::BaseDirectory, AppHandle, Manager};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Migrate error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
}

#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
    #[allow(unused)]
    pub path: PathBuf,
    #[allow(unused)]
    pub last_connected: std::time::SystemTime,
}

impl Database {
    pub async fn new(db_path: PathBuf, app_handle: AppHandle) -> Result<Self, DatabaseError> {
        // Create parent directories if they don't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db_url = format!("{}", db_path.display());

        // Create database if it doesn't exist
        tracing::debug!("Checking if DB exists...{:?}", db_url);
        if Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            tracing::debug!("DB exists");
        } else {
            tracing::debug!("DB does not exist, creating...");
            match Sqlite::create_database(&db_url).await {
                Ok(_) => {
                    tracing::debug!("DB created");
                }
                Err(e) => {
                    tracing::debug!("Error creating DB: {:?}", e);
                }
            }
        }

        // Create connection pool with refined settings
        tracing::debug!("Creating connection pool...");
        let pool = SqlitePoolOptions::new()
            .acquire_timeout(Duration::from_secs(5))
            .max_connections(10)
            .after_connect(|conn, _| {
                Box::pin(async move {
                    let conn = &mut *conn;
                    // Enable WAL mode
                    sqlx::query("PRAGMA journal_mode=WAL")
                        .execute(&mut *conn)
                        .await?;
                    // Set busy timeout
                    sqlx::query("PRAGMA busy_timeout=5000")
                        .execute(&mut *conn)
                        .await?;
                    // Enable foreign keys and triggers
                    sqlx::query("PRAGMA foreign_keys = ON;")
                        .execute(&mut *conn)
                        .await?;
                    sqlx::query("PRAGMA recursive_triggers = ON;")
                        .execute(&mut *conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(&format!("{}?mode=rwc", db_url))
            .await?;

        // Run migrations
        tracing::debug!("Running migrations...");
        let migrations_path = app_handle
            .path()
            .resolve("db_migrations", BaseDirectory::Resource)?;

        sqlx::migrate::Migrator::new(migrations_path)
            .await?
            .run(&pool)
            .await?;

        Ok(Self {
            pool,
            path: db_path,
            last_connected: std::time::SystemTime::now(),
        })
    }

    pub async fn delete_all_data(&self) -> Result<(), DatabaseError> {
        let mut txn = self.pool.begin().await?;

        // Disable foreign key constraints temporarily
        sqlx::query("PRAGMA foreign_keys = OFF")
            .execute(&mut *txn)
            .await?;

        // Delete data in reverse order of dependencies
        sqlx::query("DELETE FROM messages")
            .execute(&mut *txn)
            .await?;
        sqlx::query("DELETE FROM invites")
            .execute(&mut *txn)
            .await?;
        sqlx::query("DELETE FROM groups").execute(&mut *txn).await?;
        sqlx::query("DELETE FROM accounts")
            .execute(&mut *txn)
            .await?;
        sqlx::query("DELETE FROM relays").execute(&mut *txn).await?;

        // Re-enable foreign key constraints
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut *txn)
            .await?;

        txn.commit().await?;
        Ok(())
    }
}
