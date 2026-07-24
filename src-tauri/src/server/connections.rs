//! 连接管理器 — 跟踪所有已连接的客户端，支持心跳检测与超时清理

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;
use tokio_tungstenite::tungstenite::Message;

use crate::protocol::DeviceType;

/// 单个客户端连接的状态
pub struct ClientConnection {
    pub device_id: String,
    pub device_type: DeviceType,
    pub session_id: String,
    pub last_ping: Instant,
    /// 通过这个 channel 把要发送给客户端的消息推给写循环任务
    pub writer_tx: mpsc::UnboundedSender<Message>,
}

#[derive(Clone)]
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, ClientConnection>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let manager = Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        };
        manager.start_heartbeat_checker();
        manager
    }

    /// 注册新连接
    pub async fn add(&self, conn: ClientConnection) {
        let id = conn.device_id.clone();
        self.connections.write().await.insert(id, conn);
    }

    /// 移除连接
    pub async fn remove(&self, device_id: &str) {
        self.connections.write().await.remove(device_id);
    }

    /// 更新心跳时间
    pub async fn update_ping(&self, device_id: &str) {
        let mut conns = self.connections.write().await;
        if let Some(conn) = conns.get_mut(device_id) {
            conn.last_ping = Instant::now();
        }
    }

    /// 广播消息给所有连接；同时剔除已断开的 writer
    pub async fn broadcast(&self, message: Message) {
        let mut dead = Vec::new();
        let conns = self.connections.read().await;
        for (id, conn) in conns.iter() {
            if conn.writer_tx.send(message.clone()).is_err() {
                dead.push(id.clone());
            }
        }
        drop(conns);
        if !dead.is_empty() {
            let mut conns = self.connections.write().await;
            for id in dead {
                conns.remove(&id);
            }
        }
    }

    /// 定向发送给指定设备
    pub async fn send_to(&self, device_id: &str, message: Message) -> Result<(), String> {
        let conns = self.connections.read().await;
        match conns.get(device_id) {
            Some(conn) => conn
                .writer_tx
                .send(message)
                .map_err(|_| format!("设备 {} 的写通道已关闭", device_id)),
            None => Err(format!("设备 {} 未连接", device_id)),
        }
    }

    /// 列出所有已连接设备 ID
    pub async fn list_devices(&self) -> Vec<(String, DeviceType)> {
        self.connections
            .read()
            .await
            .iter()
            .map(|(id, c)| (id.clone(), c.device_type.clone()))
            .collect()
    }

    /// 心跳检查：90 秒未收到 ping 就断开
    fn start_heartbeat_checker(&self) {
        let connections = self.connections.clone();
        tokio::spawn(async move {
            // 错峰：27s 周期，避免与 30s 心跳同步
            let mut ticker = interval(Duration::from_secs(30));
            loop {
                ticker.tick().await;
                let mut conns = connections.write().await;
                let now = Instant::now();
                let dead: Vec<String> = conns
                    .iter()
                    .filter(|(_, c)| now.duration_since(c.last_ping) > Duration::from_secs(90))
                    .map(|(id, _)| id.clone())
                    .collect();
                for id in dead {
                    log::info!("心跳超时，断开: {}", id);
                    conns.remove(&id);
                }
            }
        });
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}
