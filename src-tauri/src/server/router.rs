//! 消息路由器 — 从 Transport 读取客户端消息，调用 AI / 会话管理，再广播响应
//!
//! Phase 0 仅做 echo + 日志，Phase 1 接入真实 AI 客户端

use tokio::sync::mpsc;

use crate::network::OutgoingMsg;
use crate::protocol::{
    timestamp_now, AiStatus, ClientToServer, MessageFrom, ServerPayload, ServerToClient,
};

/// 路由器任务：持有 incoming_rx 与 outgoing_tx，循环处理客户端消息
pub async fn run(
    mut incoming_rx: mpsc::UnboundedReceiver<ClientToServer>,
    outgoing_tx: mpsc::UnboundedSender<OutgoingMsg>,
) {
    log::info!("消息路由器启动");
    while let Some(msg) = incoming_rx.recv().await {
        if let Some(reply) = handle(msg) {
            let _ = outgoing_tx.send(OutgoingMsg {
                target: None, // Phase 0：广播给所有设备
                message: reply,
            });
        }
    }
    log::info!("消息路由器退出（transport 已停止）");
}

/// Phase 0：echo + 状态识别，验证链路打通
/// Phase 1：替换为 AI 调用 + 会话存储 + 摘要生成
fn handle(msg: ClientToServer) -> Option<ServerToClient> {
    match msg {
        ClientToServer::Message {
            session_id,
            device_id,
            device_type,
            seq,
            payload,
            timestamp: _,
        } => {
            log::info!(
                "收到消息: session={}/{:?} device={:?} seq={:?} payload={:?}",
                session_id,
                device_type,
                device_id,
                seq,
                payload
            );

            let echo = payload.text.unwrap_or_default();
            let summary: String = echo.chars().take(30).collect();
            let reply = ServerToClient::Message {
                session_id,
                seq: seq.unwrap_or(0).max(1),
                from: MessageFrom::Ai,
                payload: ServerPayload {
                    content: Some(format!("[echo] {}", echo)),
                    summary: Some(summary),
                    status: Some(AiStatus::Done),
                    error: None,
                },
                timestamp: timestamp_now(),
            };
            Some(reply)
        }
        ClientToServer::Join {
            session_id,
            device_id,
            device_type,
            timestamp: _,
        } => {
            log::info!(
                "设备加入: session={} device={:?} type={:?}",
                session_id,
                device_id,
                device_type
            );
            None
        }
        ClientToServer::Heartbeat {
            device_id,
            timestamp: _,
        } => {
            log::debug!("heartbeat from {}", device_id);
            None
        }
    }
}
