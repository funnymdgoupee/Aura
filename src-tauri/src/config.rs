use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const DEFAULT_PORT: u16 = 8765;

/// 默认 base_url 指向 DeepSeek 官方接口（兼容 OpenAI 协议）
/// 用户可在前端改成任意 OpenAI 兼容服务
const DEFAULT_AI_BASE_URL: &str = "https://api.deepseek.com/v1";
const DEFAULT_AI_MODEL: &str = "deepseek-chat";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// OpenAI 兼容 API 的 base_url
    /// 例：https://api.deepseek.com/v1 / https://api.openai.com/v1
    ///     https://open.bigmodel.cn/api/paas/v4 / https://api.moonshot.cn/v1
    pub ai_base_url: String,
    pub ai_api_key: String,
    /// 模型名，例：deepseek-chat / gpt-4o / glm-4.6 / kimi-k2
    pub ai_model: String,
    pub server_port: u16,
    pub connection_mode: ConnectionMode,
    pub relay_server_url: String,
    pub relay_room_id: String,
    pub relay_secret_key: String,
    /// 聊天历史 Markdown 文件存放目录
    /// None = 默认 `<app_data_dir>/sessions/`
    /// Some(path) = 用户自选目录（iCloud / Dropbox / 外置盘等）
    pub storage_dir: Option<String>,
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
            ai_base_url: DEFAULT_AI_BASE_URL.to_string(),
            ai_api_key: String::new(),
            ai_model: DEFAULT_AI_MODEL.to_string(),
            server_port: DEFAULT_PORT,
            connection_mode: ConnectionMode::Lan,
            relay_server_url: String::new(),
            relay_room_id: String::new(),
            relay_secret_key: String::new(),
            storage_dir: None,
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
                    "ai_base_url" => cfg.ai_base_url = v,
                    "ai_api_key" => cfg.ai_api_key = v,
                    "ai_model" => cfg.ai_model = v,
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
                    "storage_dir" => cfg.storage_dir = if v.is_empty() { None } else { Some(v) },
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
        db.upsert_config("ai_base_url", &self.ai_base_url).await?;
        db.upsert_config("ai_api_key", &self.ai_api_key).await?;
        db.upsert_config("ai_model", &self.ai_model).await?;
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
        db.upsert_config("storage_dir", self.storage_dir.as_deref().unwrap_or("")).await?;
        Ok(())
    }
}
