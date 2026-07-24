//! Tauri 命令层 — 暴露给前端的 Rust 函数

use std::path::PathBuf;

use serde::Serialize;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::config::{AppConfig, ConnectionMode};
use crate::network::lan::LanTransport;
use crate::network::relay::RelayTransport;
use crate::network::{TransportHandle, TransportStatus};
use crate::pairing;
use crate::protocol::{timestamp_now, ClientPayload, ClientToServer, DeviceType};
use crate::session::SessionMeta;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub mode: String,
    pub status: String,
    pub port: Option<u16>,
    pub server_url: Option<String>,
    pub error: Option<String>,
}

/// 启动传输 + 启动 router 任务
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

    // 把 incoming_rx 移交给 router
    let incoming_rx = handle.take_incoming();

    // 创建 Mac 前端 → router 的本地通道
    let (local_tx, local_rx) =
        tokio::sync::mpsc::unbounded_channel::<ClientToServer>();

    // 启动 router 任务
    if let Some(incoming_rx) = incoming_rx {
        let outgoing_tx = handle.outgoing_tx.clone();
        let config_arc = state.config.clone();
        let sessions = state.sessions.clone();
        let app_for_router = app.clone();
        tokio::spawn(crate::server::router::run(
            incoming_rx,
            local_rx,
            outgoing_tx,
            app_for_router,
            config_arc,
            sessions,
        ));
    }

    // 把 local_tx 存到 AppState，前端发送消息时用
    *state.local_tx.lock().await = Some(local_tx);

    *transport_guard = Some(handle);
    Ok(())
}

/// 停止传输
#[tauri::command]
pub async fn stop_transport(state: State<'_, AppState>) -> Result<(), String> {
    let mut transport_guard = state.transport.lock().await;
    if let Some(mut handle) = transport_guard.take() {
        handle.stop().await;
    }
    // 同时清空 local_tx，前端再发会报"router 未启动"
    *state.local_tx.lock().await = None;
    Ok(())
}

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

/// Phase 0 调试用：直接广播（不经 router / 不调 AI）
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
        timestamp: timestamp_now(),
    };
    handle.send(None, reply).map_err(|e| e.to_string())?;
    Ok(())
}

/// Phase 1：前端发消息走 router → 调 AI → 广播响应
#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    text: String,
    session_id: Option<String>,
) -> Result<(), String> {
    let local_tx_guard = state.local_tx.lock().await;
    let Some(tx) = local_tx_guard.as_ref() else {
        return Err("router 未启动，请先点击启动服务".into());
    };
    let session = session_id.unwrap_or_else(|| "mac-local".to_string());
    let msg = ClientToServer::Message {
        session_id: session,
        device_id: "mac-local".into(),
        device_type: DeviceType::Iphone, // 前端按 iphone 走，watch_target=false
        seq: None,
        payload: ClientPayload {
            text: Some(text),
            ..Default::default()
        },
        timestamp: timestamp_now(),
    };
    tx.send(msg).map_err(|_| "router 已停止".to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn list_sessions(state: State<'_, AppState>) -> Result<Vec<SessionMeta>, String> {
    Ok(state.sessions.list_sessions().await)
}

#[tauri::command]
pub async fn clear_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    state.sessions.clear(&session_id).await;
    Ok(())
}

/// 弹出文件夹选择对话框 — 返回所选路径（取消则 None）
#[tauri::command]
pub async fn pick_storage_folder(app: AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title("选择 Aura 会话存储位置")
        .pick_folder(move |result| {
            let _ = tx.send(match result {
                Ok(Some(FilePath::Path(p))) => Some(p.to_string_lossy().to_string()),
                _ => None,
            });
        });
    let result = rx.await.map_err(|_| "对话框错误".to_string())?;
    Ok(result)
}

/// 用户在设置里改了存储位置后，前端调此命令切换 SessionStore 的 base_dir
#[tauri::command]
pub async fn set_storage_dir(
    app: AppHandle,
    state: State<'_, AppState>,
    dir: Option<String>,
) -> Result<(), String> {
    let path = match dir {
        Some(d) if !d.is_empty() => PathBuf::from(d),
        _ => app
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("sessions"),
    };
    state.sessions.set_base_dir(path).await;
    Ok(())
}
