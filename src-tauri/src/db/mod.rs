//! SQLite 存储层
//!
//! Phase 0：仅初始化 schema + 提供 config 表 CRUD（其他表骨架）
//! Phase 1：补齐 sessions/messages/connected_devices/pairing_requests CRUD

use std::path::PathBuf;
use std::sync::Arc;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite};

pub struct Db {
    pool: Arc<Pool<Sqlite>>,
}

impl Db {
    pub async fn open(db_path: PathBuf) -> anyhow::Result<Self> {
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }
        let opts = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(opts)
            .await?;
        Self::init_schema(&pool).await?;
        Ok(Self { pool: Arc::new(pool) })
    }

    async fn init_schema(pool: &Pool<Sqlite>) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT,
                created_at INTEGER,
                updated_at INTEGER,
                model TEXT DEFAULT 'deepseek-chat',
                system_prompt TEXT
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,
                role TEXT CHECK(role IN ('user', 'assistant', 'system')),
                content TEXT,
                summary TEXT,
                seq INTEGER,
                created_at INTEGER,
                attachments JSON
            );

            CREATE TABLE IF NOT EXISTS connected_devices (
                id TEXT PRIMARY KEY,
                type TEXT CHECK(type IN ('iphone', 'watch')),
                session_id TEXT,
                last_seen INTEGER,
                ip_address TEXT,
                is_paired BOOLEAN DEFAULT FALSE
            );

            CREATE TABLE IF NOT EXISTS pairing_requests (
                id TEXT PRIMARY KEY,
                device_id TEXT,
                device_type TEXT,
                session_id TEXT,
                status TEXT CHECK(status IN ('pending', 'approved', 'rejected')),
                created_at INTEGER
            );

            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT
            );
            "#,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_config(&self, key: &str, value: &str) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO config (key, value) VALUES (?, ?) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(key)
        .bind(value)
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }

    pub async fn list_config(&self) -> anyhow::Result<Vec<(String, String)>> {
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT key, value FROM config")
                .fetch_all(self.pool.as_ref())
                .await?;
        Ok(rows)
    }
}
