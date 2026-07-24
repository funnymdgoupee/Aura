//! 局域网模式：Mac 作为 WebSocket 服务器，iPhone 直连
//!
//! 监听 0.0.0.0:{port}，接受连接，按设备 ID 跟踪每个客户端，
//! 30s 心跳 / 90s 超时清理，断线重连由客户端负责

use std::sync::{Arc, RwLock};
use std::time::Instant;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;

use crate::network::{OutgoingMsg, TransportHandle, TransportStatus};
use crate::protocol::{ClientToServer, DeviceType};
use crate::server::ConnectionManager;

pub struct LanTransport {
    port: u16,
    status: Arc<RwLock<TransportStatus>>,
}

impl LanTransport {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            status: Arc::new(RwLock::new(TransportStatus::Stopped)),
        }
    }
}

#[async_trait]
impl super::Transport for LanTransport {
    async fn start(&self) -> Result<TransportHandle, anyhow::Error> {
        let listener = TcpListener::bind(("0.0.0.0", self.port)).await?;
        let bound_port = listener.local_addr()?.port();

        let (outgoing_tx, outgoing_rx) = mpsc::unbounded_channel::<OutgoingMsg>();
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel::<ClientToServer>();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        *self.status.write().unwrap() = TransportStatus::Listening { port: bound_port };

        let status = self.status.clone();
        let join_handle: JoinHandle<()> = tokio::spawn(async move {
            run_server(listener, outgoing_rx, incoming_tx, shutdown_rx, status).await;
        });

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

async fn run_server(
    listener: TcpListener,
    mut outgoing_rx: mpsc::UnboundedReceiver<OutgoingMsg>,
    incoming_tx: mpsc::UnboundedSender<ClientToServer>,
    mut shutdown_rx: oneshot::Receiver<()>,
    status: Arc<RwLock<TransportStatus>>,
) {
    let manager = ConnectionManager::new();

    loop {
        tokio::select! {
            _ = &mut shutdown_rx => {
                log::info!("LanTransport: 收到关闭信号");
                break;
            }
            Ok((stream, addr)) = listener.accept() => {
                let manager = manager.clone();
                let incoming_tx = incoming_tx.clone();
                tokio::spawn(handle_connection(stream, addr, manager, incoming_tx));
            }
            Some(outgoing) = outgoing_rx.recv() => {
                let json = match serde_json::to_string(&outgoing.message) {
                    Ok(s) => s,
                    Err(e) => {
                        log::error!("序列化 outgoing 消息失败: {}", e);
                        continue;
                    }
                };
                let msg = Message::Text(json);
                if let Some(target) = outgoing.target {
                    if let Err(e) = manager.send_to(&target, msg).await {
                        log::warn!("发送给 {} 失败: {}", target, e);
                    }
                } else {
                    manager.broadcast(msg).await;
                }
            }
        }
    }

    *status.write().unwrap() = TransportStatus::Stopped;
    log::info!("LanTransport 服务器已停止");
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    addr: std::net::SocketAddr,
    manager: ConnectionManager,
    incoming_tx: mpsc::UnboundedSender<ClientToServer>,
) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            log::warn!("握手失败 {}: {}", addr, e);
            return;
        }
    };

    let (mut ws_writer, mut ws_reader) = ws_stream.split();
    let (writer_tx, mut writer_rx) = mpsc::unbounded_channel::<Message>();

    // 写循环：从 writer_rx 取消息发到 socket
    let writer_task = tokio::spawn(async move {
        while let Some(msg) = writer_rx.recv().await {
            if ws_writer.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut device_id: Option<String> = None;

    // 读循环
    loop {
        match ws_reader.next().await {
            Some(Ok(msg)) => match msg {
                Message::Text(text) => {
                    let parsed: Result<ClientToServer, _> = serde_json::from_str(&text);
                    match parsed {
                        Ok(parsed_msg) => {
                            // 首次消息决定 device_id / type，注册到连接管理器
                            if device_id.is_none() {
                                let (did, dt, sid) = match &parsed_msg {
                                    ClientToServer::Message { device_id, device_type, session_id, .. } => {
                                        (device_id.clone(), device_type.clone(), session_id.clone())
                                    }
                                    ClientToServer::Join { device_id, device_type, session_id, .. } => {
                                        (device_id.clone(), device_type.clone(), session_id.clone())
                                    }
                                    ClientToServer::Heartbeat { device_id, .. } => {
                                        (device_id.clone(), DeviceType::Iphone, String::new())
                                    }
                                };

                                device_id = Some(did.clone());
                                let conn = crate::server::connections::ClientConnection {
                                    device_id: did.clone(),
                                    device_type: dt,
                                    session_id: sid,
                                    last_ping: Instant::now(),
                                    writer_tx: writer_tx.clone(),
                                };
                                manager.add(conn).await;
                                log::info!("新设备连接: {} from {}", did, addr);
                            }

                            // 心跳：更新 ping 时间，不转发给路由
                            if let ClientToServer::Heartbeat { device_id, .. } = &parsed_msg {
                                manager.update_ping(device_id).await;
                            } else {
                                let _ = incoming_tx.send(parsed_msg);
                            }
                        }
                        Err(e) => {
                            log::warn!("解析消息失败 from {}: {} | raw={}", addr, e, text);
                        }
                    }
                }
                Message::Binary(_) => {
                    log::debug!("收到二进制消息 from {}", addr);
                }
                Message::Close(_) => {
                    log::info!("连接关闭: {}", addr);
                    break;
                }
                Message::Ping(_) | Message::Pong(_) => {
                    // WebSocket 层的 ping/pong 由 tungstenite 自动处理
                }
                _ => {}
            },
            Some(Err(e)) => {
                log::warn!("读取错误 from {}: {}", addr, e);
                break;
            }
            None => break,
        }
    }

    // 清理
    if let Some(did) = device_id {
        manager.remove(&did).await;
    }
    drop(writer_tx);
    let _ = writer_task.await;
}
