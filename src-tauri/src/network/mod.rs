//! 网络传输抽象层
//!
//! Transport trait 屏蔽"局域网直连"和"用户自备中继服务器"两种模式的差异。
//! 业务层（消息路由、AI 调用、会话管理）只与 TransportHandle 交互，
//! 不感知底层是 server 还是 client 角色。

pub mod lan;
pub mod relay;

use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::protocol::{ClientToServer, ServerToClient};

/// 启动后返回的传输端点句柄
///
/// - `outgoing_tx`：业务层通过它推送待发送消息（指定设备或广播）；可 Clone
/// - `incoming_rx`：业务层从这里读取客户端发来的消息；Option 以便 take 出来交给路由器任务
/// - `shutdown_tx`：发送 () 即可停止传输
/// - `join_handle`：传输任务句柄，停止后可 await 完成清理
pub struct TransportHandle {
    pub outgoing_tx: mpsc::UnboundedSender<OutgoingMsg>,
    pub incoming_rx: Option<mpsc::UnboundedReceiver<ClientToServer>>,
    pub shutdown_tx: Option<oneshot::Sender<()>>,
    pub join_handle: Option<JoinHandle<()>>,
}

impl TransportHandle {
    /// 向指定设备发送消息；device_id 为 None 时广播给所有已连接客户端
    pub fn send(
        &self,
        target: Option<String>,
        message: ServerToClient,
    ) -> Result<(), String> {
        self.outgoing_tx
            .send(OutgoingMsg { target, message })
            .map_err(|_| "transport 已停止".to_string())
    }

    /// 取出 incoming_rx 交给路由器任务（只能调用一次）
    pub fn take_incoming(
        &mut self,
    ) -> Option<mpsc::UnboundedReceiver<ClientToServer>> {
        self.incoming_rx.take()
    }

    /// 停止传输端点
    pub async fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.await;
        }
    }
}

#[derive(Debug, Clone)]
pub struct OutgoingMsg {
    /// None = 广播
    pub target: Option<String>,
    pub message: ServerToClient,
}

#[derive(Debug, Clone)]
pub enum TransportStatus {
    Stopped,
    Listening { port: u16 },
    Connected { server_url: String },
    Error(String),
}

/// Transport 是一个工厂：调用 start() 后返回运行中的 TransportHandle
#[async_trait]
pub trait Transport: Send + Sync {
    async fn start(&self) -> Result<TransportHandle, anyhow::Error>;
    fn status(&self) -> TransportStatus;
}
