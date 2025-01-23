use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("Migrate error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
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
    pub async fn new(db_path: PathBuf) -> Result<Self, DatabaseError> {
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
            .max_connections(2)
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
                    // Mobile-friendly settings
                    sqlx::query("PRAGMA synchronous=NORMAL")
                        .execute(&mut *conn)
                        .await?;
                    sqlx::query("PRAGMA page_size=4096")
                        .execute(&mut *conn)
                        .await?;
                    sqlx::query("PRAGMA cache_size=-2000") // 2MB cache
                        .execute(&mut *conn)
                        .await?;
                    sqlx::query("PRAGMA temp_store=MEMORY")
                        .execute(&mut *conn)
                        .await?;
                    Ok(())
                })
            })
            .connect(&format!("{}?mode=rwc", db_url))
            .await?;

        // Run migrations
        tracing::debug!("Running migrations...");
        let migrations_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("db_migrations");
        sqlx::migrate::Migrator::new(migrations_path)
            .await?
            .run(&pool)
            .await?;

        // Enable foreign keys and triggers
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA recursive_triggers = ON;")
            .execute(&pool)
            .await?;

        Ok(Self {
            pool,
            path: db_path,
            last_connected: std::time::SystemTime::now(),
        })
    }

    pub async fn delete_all_data(&self) -> Result<(), DatabaseError> {
        let mut txn = self.pool.begin().await?;
        sqlx::query("DELETE FROM messages_fts")
            .execute(&mut *txn)
            .await?;
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
        sqlx::query("DELETE FROM active_account")
            .execute(&mut *txn)
            .await?;
        txn.commit().await?;
        Ok(())
    }
}
