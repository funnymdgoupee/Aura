//! 消息路由器 — 从 WS / 前端读取消息，调用 AI 客户端，广播响应
//!
//! 双入口：
//!   - incoming_rx：WebSocket 客户端发来的消息（iPhone / Watch）
//!   - local_rx：Mac 前端发来的消息（通过 Tauri 命令 send_message）
//!
//! 双出口：
//!   - outgoing_tx：广播给所有已连接 WS 客户端
//!   - app_handle.emit(...)：推送给 Mac 前端
//!
//! Phase 1.0：非流式，等 AI 完整响应后广播一次
//! Phase 1.1：改 chat_stream，每收到 chunk 广播一次

use std::sync::Arc;

use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex};

use crate::ai::{AiClient, ChatMessage, ChatRequest};
use crate::config::AppConfig;
use crate::network::OutgoingMsg;
use crate::protocol::{
    timestamp_now, AiStatus, ClientToServer, DeviceType, MessageFrom, ServerPayload,
    ServerToClient,
};
use crate::session::SessionStore;

pub async fn run(
    mut incoming_rx: mpsc::UnboundedReceiver<ClientToServer>,
    mut local_rx: mpsc::UnboundedReceiver<ClientToServer>,
    outgoing_tx: mpsc::UnboundedSender<OutgoingMsg>,
    app_handle: AppHandle,
    config: Arc<Mutex<AppConfig>>,
    sessions: SessionStore,
) {
    log::info!("消息路由器启动");

    loop {
        let msg = tokio::select! {
            Some(msg) = incoming_rx.recv() => msg,
            Some(msg) = local_rx.recv() => msg,
            else => break,
        };
        process_message(msg, &outgoing_tx, &app_handle, &config, &sessions).await;
    }

    log::info!("消息路由器退出（两端通道均关闭）");
}

async fn process_message(
    msg: ClientToServer,
    outgoing_tx: &mpsc::UnboundedSender<OutgoingMsg>,
    app_handle: &AppHandle,
    config: &Arc<Mutex<AppConfig>>,
    sessions: &SessionStore,
) {
    let ClientToServer::Message {
        session_id,
        device_id,
        device_type,
        seq: _,
        payload,
        timestamp: _,
    } = msg
    else {
        // Join / Heartbeat 不走 AI
        return;
    };

    let Some(text) = payload.text else {
        return;
    };
    if text.trim().is_empty() {
        return;
    }

    let watch_target = matches!(device_type, DeviceType::Watch);

    // 1. 把用户消息追加到会话历史
    sessions
        .append(
            &session_id,
            ChatMessage {
                role: "user".into(),
                content: text.clone(),
            },
        )
        .await;

    // 2. 广播 Thinking 状态给 WS + 前端
    let thinking_status = ServerToClient::Status {
        session_id: session_id.clone(),
        status: AiStatus::Thinking,
        error: None,
        timestamp: timestamp_now(),
    };
    let _ = outgoing_tx.send(OutgoingMsg {
        target: None,
        message: thinking_status.clone(),
    });
    let _ = app_handle.emit("ai_status", &thinking_status);

    // 3. 读配置构造 AI 客户端
    let cfg = config.lock().await.clone();
    let ai_client = AiClient::new(cfg.ai_base_url.clone(), cfg.ai_api_key.clone());

    let history = sessions.get_or_create(&session_id).await;

    let req = ChatRequest {
        session_id: session_id.clone(),
        messages: history,
        model: cfg.ai_model.clone(),
        watch_target,
    };

    // 4. 调用 AI
    match ai_client.chat(req).await {
        Ok(resp) => {
            // 把 AI 回复追加到历史
            sessions
                .append(
                    &session_id,
                    ChatMessage {
                        role: "assistant".into(),
                        content: resp.content.clone(),
                    },
                )
                .await;

            let reply = ServerToClient::Message {
                session_id: session_id.clone(),
                seq: next_seq(),
                from: MessageFrom::Ai,
                payload: ServerPayload {
                    content: Some(resp.content.clone()),
                    summary: resp.summary.clone(),
                    status: Some(AiStatus::Done),
                    error: None,
                },
                timestamp: timestamp_now(),
            };

            // 广播给 WS
            let _ = outgoing_tx.send(OutgoingMsg {
                target: None,
                message: reply.clone(),
            });
            // 推送给前端
            let _ = app_handle.emit("ai_message", &reply);

            log::info!(
                "AI 回复 session={} device={} bytes={}",
                session_id,
                device_id,
                resp.content.len()
            );
        }
        Err(e) => {
            log::error!("AI 调用失败: {}", e);

            let err_msg = ServerToClient::Error {
                session_id: session_id.clone(),
                code: "ai_error".into(),
                message: e.to_string(),
                timestamp: timestamp_now(),
            };

            let _ = outgoing_tx.send(OutgoingMsg {
                target: None,
                message: err_msg.clone(),
            });
            let _ = app_handle.emit("ai_error", &err_msg);
        }
    }
}

/// 简单的全局递增序号 — 用时间戳毫秒，足够 Phase 1 用
/// Phase 1.1 改为 per-session atomic counter
fn next_seq() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
