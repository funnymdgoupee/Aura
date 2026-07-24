//! 会话内存存储 — 维护多轮对话历史
//!
//! Phase 1.0：纯内存，进程重启即丢
//! Phase 1.5：接入 SQLite 持久化

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::ai::ChatMessage;

#[derive(Clone)]
pub struct SessionStore {
    inner: Arc<Mutex<HashMap<String, Vec<ChatMessage>>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_or_create(&self, session_id: &str) -> Vec<ChatMessage> {
        let mut inner = self.inner.lock().await;
        inner
            .entry(session_id.to_string())
            .or_insert_with(Vec::new)
            .clone()
    }

    pub async fn append(&self, session_id: &str, message: ChatMessage) {
        let mut inner = self.inner.lock().await;
        inner
            .entry(session_id.to_string())
            .or_insert_with(Vec::new)
            .push(message);
    }

    pub async fn list_sessions(&self) -> Vec<String> {
        self.inner.lock().await.keys().cloned().collect()
    }

    pub async fn clear(&self, session_id: &str) {
        self.inner.lock().await.remove(session_id);
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}
