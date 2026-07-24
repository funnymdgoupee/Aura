//! 用户自备中继服务器模式：Mac 作为客户端连接用户 VPS 上的 WebSocket 中继
//!
//! Phase 4 实现。当前为骨架，预留接口确保后期切换时业务代码不动。

use std::sync::Arc;
use std::sync::RwLock;

use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::network::{OutgoingMsg, TransportHandle, TransportStatus};
use crate::protocol::ClientToServer;

pub struct RelayTransport {
    server_url: String,
    room_id: String,
    secret_key: String,
    status: Arc<RwLock<TransportStatus>>,
}

impl RelayTransport {
    pub fn new(server_url: String, room_id: String, secret_key: String) -> Self {
        Self {
            server_url,
            room_id,
            secret_key,
            status: Arc::new(RwLock::new(TransportStatus::Stopped)),
        }
    }
}

#[async_trait]
impl super::Transport for RelayTransport {
    async fn start(&self) -> Result<TransportHandle, anyhow::Error> {
        // Phase 4 实现要点：
        // 1. 用 tokio_tungstenite::connect_async 连到 self.server_url?room=&key=
        // 2. 注册房间，开始心跳（30s ping）和断线重连（指数退避 1s→30s）
        // 3. 读循环：服务器转发的消息塞进 incoming_tx
        // 4. 写循环：从 outgoing_rx 取消息，加密后发给服务器
        // 5. 业务层加密：Mac 与 iPhone 在配对时协商共享密钥，
        //    所有业务消息在 Transport 层加密后才发送，中继服务器只看密文

        let _ = (self.room_id.as_str(), self.secret_key.as_str());

        let (outgoing_tx, _outgoing_rx) = mpsc::unbounded_channel::<OutgoingMsg>();
        let (_incoming_tx, incoming_rx) = mpsc::unbounded_channel::<ClientToServer>();
        let (shutdown_tx, _shutdown_rx) = oneshot::channel::<()>();

        *self.status.write().unwrap() = TransportStatus::Error(
            "RelayTransport 尚未实现，请在 Phase 4 完成".to_string(),
        );

        // 占位：返回一个空 handle，调用方会立即发现无法使用
        let join_handle: JoinHandle<()> = tokio::spawn(async move {
            // 故意不做任何事 — Phase 4 在这里填实现
        });

        // 恢复 status 字段以便调用方能正确判断
        *self.status.write().unwrap() = TransportStatus::Error(
            "RelayTransport 尚未实现".to_string(),
        );

        Ok(TransportHandle {
            outgoing_tx,
            incoming_rx,
            shutdown_tx: Some(shutdown_tx),
            join_handle: Some(join_handle),
        })
    }

    fn status(&self) -> TransportStatus {
        self.status.read().unwrap().clone()
    }
}
