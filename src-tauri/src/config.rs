use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const DEFAULT_PORT: u16 = 8765;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub deepseek_api_key: String,
    pub server_port: u16,
    pub connection_mode: ConnectionMode,
    pub relay_server_url: String,
    pub relay_room_id: String,
    pub relay_secret_key: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionMode {
    Lan,
    Relay,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            deepseek_api_key: String::new(),
            server_port: DEFAULT_PORT,
            connection_mode: ConnectionMode::Lan,
            relay_server_url: String::new(),
            relay_room_id: String::new(),
            relay_secret_key: String::new(),
        }
    }
}

impl AppConfig {
    /// Load from SQLite config table; fall back to default on missing/error.
    pub async fn load_or_default(app: &AppHandle) -> anyhow::Result<Self> {
        let db = app.try_state::<crate::db::Db>();
        let Some(db) = db else {
            return Ok(Self::default());
        };
        let mut cfg = Self::default();
        if let Ok(rows) = db.list_config().await {
            for (k, v) in rows {
                match k.as_str() {
                    "deepseek_api_key" => cfg.deepseek_api_key = v,
                    "server_port" => cfg.server_port = v.parse().unwrap_or(DEFAULT_PORT),
                    "connection_mode" => {
                        cfg.connection_mode = match v.as_str() {
                            "relay" => ConnectionMode::Relay,
                            _ => ConnectionMode::Lan,
                        }
                    }
                    "relay_server_url" => cfg.relay_server_url = v,
                    "relay_room_id" => cfg.relay_room_id = v,
                    "relay_secret_key" => cfg.relay_secret_key = v,
                    _ => {}
                }
            }
        }
        Ok(cfg)
    }

    pub async fn save(&self, app: &AppHandle) -> anyhow::Result<()> {
        let db = app.try_state::<crate::db::Db>();
        let Some(db) = db else {
            return Ok(());
        };
        db.upsert_config("deepseek_api_key", &self.deepseek_api_key).await?;
        db.upsert_config("server_port", &self.server_port.to_string()).await?;
        db.upsert_config(
            "connection_mode",
            match self.connection_mode {
                ConnectionMode::Lan => "lan",
                ConnectionMode::Relay => "relay",
            },
        ).await?;
        db.upsert_config("relay_server_url", &self.relay_server_url).await?;
        db.upsert_config("relay_room_id", &self.relay_room_id).await?;
        db.upsert_config("relay_secret_key", &self.relay_secret_key).await?;
        Ok(())
    }
}
