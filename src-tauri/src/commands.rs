//! Tauri 命令层 — 暴露给前端的 Rust 函数
//!
//! 前端通过 `invoke('command_name', args)` 调用

use serde::Serialize;
use tauri::{AppHandle, State};

use crate::config::{AppConfig, ConnectionMode};
use crate::network::lan::LanTransport;
use crate::network::relay::RelayTransport;
use crate::network::{TransportHandle, TransportStatus};
use crate::pairing;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub mode: String,
    pub status: String,
    pub port: Option<u16>,
    pub server_url: Option<String>,
    pub error: Option<String>,
}

/// 启动传输（按当前配置的 connection_mode 分流）
#[tauri::command]
pub async fn start_with_mode(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut transport_guard = state.transport.lock().await;
    if transport_guard.is_some() {
        return Err("已有传输在运行，请先停止".into());
    }

    let config = state.config.lock().await.clone();
    let mut handle: TransportHandle = match config.connection_mode {
        ConnectionMode::Lan => {
            let t = LanTransport::new(config.server_port);
            t.start().await.map_err(|e| e.to_string())?
        }
        ConnectionMode::Relay => {
            if config.relay_server_url.is_empty() {
                return Err("未配置中继服务器地址".into());
            }
            let t = RelayTransport::new(
                config.relay_server_url.clone(),
                config.relay_room_id.clone(),
                config.relay_secret_key.clone(),
            );
            t.start().await.map_err(|e| e.to_string())?
        }
    };

    // 把 incoming_rx 移交给路由器任务
    if let Some(incoming_rx) = handle.take_incoming() {
        let outgoing_tx = handle.outgoing_tx.clone();
        tokio::spawn(crate::server::router::run(incoming_rx, outgoing_tx));
    }

    *transport_guard = Some(handle);
    let _ = app;
    Ok(())
}

/// 停止传输
#[tauri::command]
pub async fn stop_transport(state: State<'_, AppState>) -> Result<(), String> {
    let mut transport_guard = state.transport.lock().await;
    if let Some(mut handle) = transport_guard.take() {
        handle.stop().await;
    }
    Ok(())
}

/// 查询传输状态
#[tauri::command]
pub async fn get_transport_status(state: State<'_, AppState>) -> Result<StatusResponse, String> {
    let config = state.config.lock().await.clone();
    let transport_guard = state.transport.lock().await;
    let running = transport_guard.is_some();

    let status = if !running {
        TransportStatus::Stopped
    } else {
        match config.connection_mode {
            ConnectionMode::Lan => TransportStatus::Listening {
                port: config.server_port,
            },
            ConnectionMode::Relay => TransportStatus::Connected {
                server_url: config.relay_server_url.clone(),
            },
        }
    };

    let (mode, status_str, port, server_url, error) = match status {
        TransportStatus::Stopped => ("stopped".into(), "stopped".into(), None, None, None),
        TransportStatus::Listening { port } => {
            ("lan".into(), "listening".into(), Some(port), None, None)
        }
        TransportStatus::Connected { server_url } => {
            ("relay".into(), "connected".into(), None, Some(server_url), None)
        }
        TransportStatus::Error(e) => ("error".into(), "error".into(), None, None, Some(e)),
    };

    Ok(StatusResponse {
        mode,
        status: status_str,
        port,
        server_url,
        error,
    })
}

/// 生成配对二维码
#[tauri::command]
pub async fn generate_pairing_qr(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let config = state.config.lock().await.clone();
    let info = pairing::generate_pairing_qr(&app, &config)?;
    Ok(serde_json::json!({
        "qr_data_url": info.qr_data_url,
        "raw_url": info.raw_url,
    }))
}

/// 切换连接模式（不立即生效，需重启传输）
#[tauri::command]
pub async fn switch_connection_mode(
    app: AppHandle,
    state: State<'_, AppState>,
    mode: String,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    config.connection_mode = match mode.as_str() {
        "lan" => ConnectionMode::Lan,
        "relay" => ConnectionMode::Relay,
        _ => return Err(format!("未知模式: {}", mode)),
    };
    config.save(&app).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(state.config.lock().await.clone())
}

#[tauri::command]
pub async fn save_config(
    app: AppHandle,
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    config.save(&app).await.map_err(|e| e.to_string())?;
    *state.config.lock().await = config;
    Ok(())
}

/// Phase 0 调试用：从前端发送测试消息，触发广播
#[tauri::command]
pub async fn send_test_message(
    state: State<'_, AppState>,
    text: String,
) -> Result<(), String> {
    let transport_guard = state.transport.lock().await;
    let Some(handle) = transport_guard.as_ref() else {
        return Err("传输未启动".into());
    };
    let reply = crate::protocol::ServerToClient::Message {
        session_id: "test".into(),
        seq: 0,
        from: crate::protocol::MessageFrom::Ai,
        payload: crate::protocol::ServerPayload {
            content: Some(format!("[test broadcast] {}", text)),
            summary: Some(text.chars().take(30).collect()),
            status: Some(crate::protocol::AiStatus::Done),
            error: None,
        },
        timestamp: crate::protocol::timestamp_now(),
    };
    handle.send(None, reply).map_err(|e| e.to_string())?;
    Ok(())
}
