# 跨平台通用 AI 助手 V2.1 架构设计文档

**文档版本**：v2.1
**更新日期**：2026-07-24
**主要变更**：基于 V2 架构，增加网络层健壮性设计、外网访问预留（用户自备中继服务器）、手表端智能摘要、二维码配对、Mac 菜单栏常驻模式

---

## 目录

1. [文档概述](#一文档概述)
2. [系统目标与核心概念](#二系统目标与核心概念)
3. [技术栈总览](#三技术栈总览)
4. [详细架构设计](#四详细架构设计)
5. [数据流与通信协议](#五数据流与通信协议)
6. [数据模型](#六数据模型)
7. [部署与运维](#七部署与运维)
8. [开发路线图](#八开发路线图)
9. [风险与应对](#九风险与应对)
10. [版本对比](#十版本对比)

---

## 一、文档概述

本文档为"跨平台通用 AI 助手"项目的系统架构设计说明 V2.1 版。基于 V2 架构，针对网络层健壮性、边缘场景体验和长期扩展性进行了关键调整。

> **核心理念**：以 Mac 为中心，Apple 设备原生互联。网络层做厚，手表层做薄，桌面层做隐，外网做预留。

### V2 → V2.1 的 5 个关键改动

| 改动 | V2 原方案 | V2.1 调整 | 理由 |
|------|-----------|-----------|------|
| **网络层保护** | 基础 WebSocket 连接 | 心跳 + 断线重连 + 消息去重 | iOS 后台断连是隐性灾难 |
| **外网访问** | 仅限局域网，WOL 进阶 | 预留"用户自备 WebSocket 中继服务器"接口 | 局域网限制会被用户当成 bug |
| **手表摘要** | 固定截断 20-30 字 | AI 主动生成摘要 + 点击展开 | 代码/路径类回复截断会丢失关键信息 |
| **设备配对** | Bonjour + 手动输 IP | 增加二维码配对作为 fallback | 手动输 IP 体验极差 |
| **Mac 常驻** | 普通窗口应用 | 菜单栏常驻 + 后台服务器 | 用户不会一直开着聊天窗口 |

---

## 二、系统目标与核心概念

### 2.1 产品定位

一款**深度整合 Apple 生态（Mac + iPhone + Apple Watch）的通用 AI 助手**。以 Mac 电脑为计算核心，iPhone 和 Apple Watch 作为便携交互终端，所有设备直连电脑，实现"电脑是大脑，手表是声控遥控器"的体验。

**核心价值**：
- 电脑端深度工作，手腕上语音指挥
- 零云端依赖，数据全在本地
- 隐私优先，API Key 仅存于 Mac Keychain

### 2.2 核心概念

| 概念 | 定义 |
|------|------|
| **桌面端（服务器）** | Mac 应用，持有用户自配置的 AI API Key（OpenAI 兼容协议），负责 AI 调用、文件读写、命令执行；可作为局域网 WebSocket 服务器接受远程指令，或作为客户端连接用户自备的中继服务器 |
| **手机端（客户端）** | iPhone 原生 App，连接 Mac（局域网直连或通过中继服务器），收发消息、拍照上传、接收推送，同时作为手表的网络代理 |
| **手表端（客户端）** | Apple Watch App，通过 WCSession 与 iPhone App 通信，间接控制 Mac；核心交互为语音输入 + AI 形象动画反馈 |
| **AI 形象** | 手表端的动态视觉反馈系统，通过动画表达 AI 的"倾听"、"思考"、"回应"、"执行"等状态 |
| **中继服务器（可选）** | 用户自备的 WebSocket 转发服务，部署在用户自己的 VPS 上，仅在"自定义服务器"模式下使用，做盲转发不解密业务数据 |

---

## 三、技术栈总览

### 3.1 平台技术选型

| 平台 | 技术选型 | 选型理由 |
|------|---------|---------|
| **桌面端 (Mac)** | **Tauri 2** (Rust + Swift/React) | 打包体积小、性能高、Rust 后端可做高效 WebSocket 服务器；支持菜单栏常驻 |
| **手机端 (iOS)** | **SwiftUI + Combine** | 原生开发，与 watchOS 共享大量代码，深度整合 Apple 生态 |
| **手表端 (watchOS)** | **SwiftUI + WatchKit** | 原生开发，与 iOS App 共享数据模型和通信逻辑 |
| **设备通信** | **WebSocket (桌面↔手机) + WCSession (手机↔手表)** | 局域网直连 + Apple 原生桥接；外网场景走用户自备中继 |
| **AI 服务** | **用户自配置 OpenAI 兼容 API** | 用户在前端填写 base_url + api_key + model，可接入 DeepSeek / OpenAI / Claude / Gemini / GLM / Qwen / Kimi / Doubao / Ollama 等任意 OpenAI 协议服务 |
| **外网访问（预留）** | **用户自备 WebSocket 中继服务器** | 用户自有 VPS 跑极简转发服务，数据端到端加密，中继只看密文 |

### 3.2 核心依赖库

| 模块 | 关键库/框架 | 用途 |
|------|-----------|------|
| 桌面端前端 | React / Vue 3 + Tailwind | UI 渲染 |
| 桌面端后端 (Rust) | `tauri::api`, `tokio`, `tokio-tungstenite`, `serde`, `qrcode` | 文件系统、WebSocket 服务器/客户端、命令执行、二维码生成 |
| 桌面端 AI 客户端 (Rust) | `reqwest` + `serde_json` + `futures-util` | 通用 OpenAI 兼容 Chat Completions 调用，支持流式 SSE |
| 桌面端 WebSocket (服务器模式) | `tokio-tungstenite` (Rust) | 局域网模式下作为 WebSocket 服务器 |
| 桌面端 WebSocket (中继客户端模式) | `tokio-tungstenite` + `tokio` 重连逻辑 | 自定义服务器模式下作为客户端连接用户中继 |
| 桌面端菜单栏 | `tauri::SystemTray` | 菜单栏常驻模式 |
| iOS 端 | SwiftUI + Combine + `URLSessionWebSocketTask` | UI + 状态管理 + WebSocket 客户端 |
| iOS 扫码 | `AVFoundation` | 二维码扫描配对 |
| watchOS 端 | SwiftUI + WatchKit + `WCSession` | UI + 与 iPhone 通信 |
| 语音识别 (watchOS) | `SFSpeechRecognizer` | 语音转文字 |
| 中继服务器（用户自备） | Node.js / Go / Rust（任选） | 极简 WebSocket 房间转发，< 200 行；后期提供开源 Docker 镜像 |

---

## 四、详细架构设计

### 4.1 整体架构图

```
┌───────────────────────────────────────────────────────────────────────────────┐
│                              网络传输抽象层                                    │
│  ┌─────────────┐  ┌─────────────────┐  ┌─────────────────────────────────┐   │
│  │ 局域网直连   │  │ 自定义中继服务器  │  │ 手动输入 IP / 二维码 (fallback) │   │
│  │ (默认模式)  │  │ (用户自备 VPS)   │  │                                 │   │
│  │ Mac=server  │  │ Mac=client       │  │                                 │   │
│  └─────────────┘  └─────────────────┘  └─────────────────────────────────┘   │
│         ↑              ↑                              ↑                       │
│         └──────────────┴──────────────────────────────┘                       │
│                              Transport 接口统一封装                            │
└───────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│ 两种连接模式拓扑                                                               │
│                                                                               │
│ 模式 A：局域网（默认）                                                         │
│ ┌──────────────┐  ws://ip:8765  ┌──────────────┐                            │
│ │ iPhone       │ ─────────────► │ Mac (服务器)  │                            │
│ │ (client)     │                │ 监听 0.0.0.0   │                            │
│ └──────────────┘                └──────────────┘                            │
│                                                                               │
│ 模式 B：自定义中继服务器（用户自备，Phase 4+）                                  │
│ ┌──────────────┐  wss://server  ┌──────────────────┐  wss://server  ┌──────┐ │
│ │ iPhone       │ ─────────────► │ 用户的中继服务器  │ ◄─────────────│ Mac  │ │
│ │ (client)     │                │ (盲转发，不解密)  │  (client)     │      │ │
│ └──────────────┘                └──────────────────┘                └──────┘ │
│ 两种模式下业务层完全一致：消息路由、AI 调用、会话管理不感知传输方式            │
└───────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌───────────────────────────────────────────────────────────────────────────────┐
│ 局域网 (WiFi / 有线) 或 用户 VPS 中继                                          │
│                                                                               │
│ ┌─────────────────────────────────────────────────────────────────────────┐   │
│ │ 桌面端 (Mac)                                                            │   │
│ │ Tauri + Rust + React/Vue                                                │   │
│ │                                                                         │   │
│ │ ┌──────────────────────────────────────────────────────────────────┐   │   │
│ │ │ 双重身份 + 菜单栏常驻 + 连接模式切换                              │   │   │
│ │ │ ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐  │   │   │
│ │ │ │ AI 客户端    │◄───┤ 消息路由器   │◄───┤ Transport 抽象层    │  │   │   │
│ │ │ │ AI 调用      │    │ (seq 去重)  │    │ Lan / Relay 实现    │  │   │   │
│ │ │ └─────────────┘    └─────────────┘    └─────────────────────┘  │   │   │
│ │ └──────────────────────────────────────────────────────────────────┘   │   │
│ │                                                                         │   │
│ │ ┌─────────────┐    ┌─────────────────┐    ┌─────────────────────┐     │   │
│ │ │ 文件系统     │    │ 连接管理器       │    │ 二维码配对生成器     │     │   │
│ │ └─────────────┘    │ (心跳/去重/广播) │    │ (含模式信息)        │     │   │
│ │ ┌─────────────┐    └─────────────────┘    └─────────────────────┘     │   │
│ │ │ 命令执行     │                           ┌─────────────────────┐     │   │
│ │ └─────────────┘                           │ SQLite 存储层        │     │   │
│ │ ┌─────────────┐                           │ (含连接模式配置)    │     │   │
│ │ │ 菜单栏常驻   │                           └─────────────────────┘     │   │
│ │ └─────────────┘                                                         │   │
│ └───────────────────────────────────────────────────────────────────────┘   │
│                                      │                                      │
│           WebSocket 连接 (两种模式透明)                                     │
│                                      │                                      │
│ ┌──────────────────────────┴──────────────────────────┐                    │
│ │                                                     │                    │
│ ▼                                                     ▼                    │
│ ┌─────────────────────────┐              ┌─────────────────────────┐       │
│ │ 手机端 (iPhone)         │              │ 手表端 (Apple Watch)    │       │
│ │ SwiftUI + Combine       │              │ SwiftUI + WatchKit      │       │
│ │                         │              │                         │       │
│ │ ┌───────────────────┐   │  WCSession   │ ┌───────────────────┐   │       │
│ │ │ Transport 抽象    │   │◄────────────►│ │ WCSession (代理)  │   │       │
│ │ │ (Lan/Relay)       │   │ (蓝牙/WiFi)  │ └───────────────────┘   │       │
│ │ │ 心跳 + 断线重连   │   │              │                         │       │
│ │ └────────┬──────────┘   │              │ ┌───────────────────┐   │       │
│ │          │              │              │ │ AI 形象动画        │   │       │
│ │ ┌────────▼──────────┐   │              │ │ (状态机 + 触觉)    │   │       │
│ │ │ UI: 聊天/拍照/扫码 │   │              │ └───────────────────┘   │       │
│ │ │ 本地缓存 SQLite    │   │              │ ┌───────────────────┐   │       │
│ │ └───────────────────┘   │              │ │ 语音识别           │   │       │
│ └─────────────────────────┘              │ └───────────────────┘   │       │
│                                          └─────────────────────────┘       │
└───────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 桌面端架构 (Mac Tauri)

桌面端兼具"AI 客户端"、"传输端点（服务器或客户端）"、"局域网发现服务"三重身份，并支持菜单栏常驻后台运行。连接模式（局域网/自定义服务器）由用户在设置面板选择，对业务层透明。

**模块划分**：

| 模块 | 路径 | 职责 |
|------|------|------|
| **UI 层** | `src-frontend/` | 聊天界面、文件拖拽、Markdown 渲染、设置面板（API Key、连接模式开关、中继服务器地址、端口） |
| **Tauri 命令层** | `src-tauri/src/commands/` | 暴露给前端的 Rust 函数（文件读写、命令执行、二维码生成、连接模式切换等） |
| **AI 客户端** | `src-tauri/src/ai/` | 通用 OpenAI 兼容 Chat Completions 调用（流式 SSE / 非流式），手表模式下要求输出 summary |
| **会话管理** | `src-tauri/src/session/` | 会话 CRUD、消息历史存储、上下文管理 |
| **网络传输抽象层** | `src-tauri/src/network/` | Transport trait + LanTransport + RelayTransport 实现，屏蔽连接模式差异 |
| **WebSocket 服务器** | `src-tauri/src/server/` | 局域网模式下监听端口、接受手机/手表连接、消息广播 |
| **WebSocket 中继客户端** | `src-tauri/src/relay_client/` | 自定义服务器模式下作为客户端连接用户中继，含心跳/重连 |
| **连接管理器** | `src-tauri/src/server/connections.rs` | 管理所有连接的客户端（deviceId → WebSocket 通道），实现心跳检测与超时清理 |
| **消息路由器** | `src-tauri/src/server/router.rs` | 解析客户端消息，转发给 AI 客户端，将响应广播给所有设备，支持 seq 去重 |
| **二维码配对** | `src-tauri/src/pairing/` | 生成包含模式信息的二维码（`aura://pair?mode=lan&...` 或 `aura://pair?mode=relay&...`） |
| **工具系统** | `src-tauri/src/tools/` | 可扩展工具集（文件读取、命令执行、网络搜索等） |
| **存储层** | `src-tauri/src/db/` | SQLite 数据库操作（会话、消息、配置、已配对设备、连接模式） |
| **菜单栏常驻** | `src-tauri/src/tray.rs` | 系统托盘菜单，支持隐藏主窗口、后台保持服务运行 |

#### 4.2.1 启动逻辑（按连接模式分流，Rust）

```rust
// src-tauri/src/main.rs
#[tokio::main]
async fn main() {
    let app = tauri::Builder::default()
        .system_tray(build_tray())
        .on_system_tray_event(handle_tray_event)
        .invoke_handler(tauri::generate_handler![
            start_with_mode,
            stop_transport,
            get_transport_status,
            generate_pairing_qr,
            switch_connection_mode
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// 启动时根据配置选择 transport 实现
async fn start_with_mode(app_handle: tauri::AppHandle) -> Result<(), Box<dyn Error>> {
    let config = load_config(&app_handle)?;
    let transport: Box<dyn Transport> = match config.connection_mode.as_str() {
        "lan" => Box::new(LanTransport::new(config.server_port)),
        "relay" => Box::new(RelayTransport::new(config.relay_server_url.clone())),
        other => return Err(format!("未知连接模式: {}", other).into()),
    };
    transport.start().await
}
```

#### 4.2.2 连接管理器（含心跳检测）

```rust
// src-tauri/src/server/connections.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, ClientConnection>>>,
}

struct ClientConnection {
    device_id: String,
    device_type: String,
    last_ping: std::time::Instant,
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let manager = Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        };
        manager.start_heartbeat_checker();
        manager
    }

    /// 心跳检查：90 秒未收到 ping 就断开
    fn start_heartbeat_checker(&self) {
        let connections = self.connections.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(30));
            loop {
                ticker.tick().await;
                let mut conns = connections.write().await;
                let now = std::time::Instant::now();
                let dead: Vec<String> = conns
                    .iter()
                    .filter(|(_, c)| now.duration_since(c.last_ping) > Duration::from_secs(90))
                    .map(|(id, _)| id.clone())
                    .collect();
                for id in dead {
                    conns.remove(&id);
                    println!("超时断开: {}", id);
                }
            }
        });
    }

    pub async fn update_ping(&self, device_id: &str) {
        let mut conns = self.connections.write().await;
        if let Some(conn) = conns.get_mut(device_id) {
            conn.last_ping = std::time::Instant::now();
        }
    }
}
```

#### 4.2.3 菜单栏常驻模式

```rust
// src-tauri/src/tray.rs
use tauri::{SystemTray, SystemTrayEvent, SystemTrayMenu, CustomMenuItem};

pub fn build_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "退出");
    let show = CustomMenuItem::new("show".to_string(), "显示主窗口");
    let status = CustomMenuItem::new("status".to_string(), "服务器运行中 ✅")
        .disabled();

    let menu = SystemTrayMenu::new()
        .add_item(status)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(show)
        .add_item(quit);

    SystemTray::new().with_menu(menu)
}

pub fn handle_tray_event(app: &tauri::AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => toggle_window(app),
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => std::process::exit(0),
            "show" => show_window(app),
            _ => {}
        },
        _ => {}
    }
}
```

#### 4.2.4 二维码配对生成（含模式信息）

```rust
// src-tauri/src/pairing.rs
use qrcode::QrCode;
use image::Luma;

#[tauri::command]
pub fn generate_pairing_qr(app_handle: tauri::AppHandle) -> Result<String, String> {
    let config = get_config(&app_handle)?;
    let session_id = generate_session_id();

    // 根据连接模式生成不同格式的 URL
    let url = match config.connection_mode.as_str() {
        "lan" => {
            let ip = get_local_ip()?;
            format!("aura://pair?mode=lan&ip={}&port={}&session={}",
                    ip, config.server_port, session_id)
        }
        "relay" => {
            // 中继模式下用户已有 room/key（自己部署的中继），二维码带这些信息
            format!("aura://pair?mode=relay&url={}&room={}&key={}",
                    urlencoding::encode(&config.relay_server_url),
                    session_id,
                    config.relay_secret_key)
        }
        _ => return Err("未配置连接模式".into()),
    };

    let code = QrCode::new(&url).map_err(|e| e.to_string())?;
    let image = code.render::<Luma<u8>>().build();

    Ok(base64_encode(image))
}
```

#### 4.2.5 网络传输抽象层（核心：屏蔽两种连接模式差异）

```rust
// src-tauri/src/network/mod.rs
use async_trait::async_trait;

#[async_trait]
pub trait Transport: Send + Sync {
    /// 启动传输端点（局域网模式=监听端口，中继模式=连接服务器）
    async fn start(&self) -> Result<(), Box<dyn Error>>;

    /// 停止传输
    async fn stop(&self) -> Result<(), Box<dyn Error>>;

    /// 发送消息给指定设备（或广播）
    async fn send(&self, device_id: Option<&str>, message: ServerToClient) -> Result<(), Box<dyn Error>>;

    /// 接收消息的通道（业务层通过这个订阅客户端消息）
    fn incoming(&self) -> mpsc::UnboundedReceiver<ClientToServer>;

    /// 当前状态
    fn status(&self) -> TransportStatus;
}

pub enum TransportStatus {
    Stopped,
    Listening { port: u16 },
    Connected { server_url: String },
    Error(String),
}

// ============ 局域网模式 ============
// Mac 作为 WebSocket 服务器，iPhone 直连
pub struct LanTransport {
    port: u16,
    manager: ConnectionManager,
    incoming_tx: mpsc::UnboundedSender<ClientToServer>,
}

#[async_trait]
impl Transport for LanTransport {
    async fn start(&self) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        let manager = self.manager.clone();
        tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                let manager = manager.clone();
                tokio::spawn(handle_connection(stream, addr, manager));
            }
        });
        Ok(())
    }
    // ... send / incoming / status 略
}

// ============ 自定义中继服务器模式 ============
// Mac 作为客户端连用户的中继服务器，iPhone 也连同一服务器
// 中继服务器做房间转发，不参与业务
pub struct RelayTransport {
    server_url: String,        // wss://user-vps.com/relay
    room_id: String,           // Mac 自动生成的房间号
    // ...
}

#[async_trait]
impl Transport for RelayTransport {
    async fn start(&self) -> Result<(), Box<dyn Error>> {
        // 连接用户的中继服务器，注册到 room
        // 心跳 + 断线重连（同 iPhone 端逻辑）
        // 服务器转发来的消息塞到 incoming channel
        // 业务层调 send() 时通过 wss 发给服务器，服务器转给同 room 的 iPhone
        todo!("Phase 4 实现")
    }
    // ...
}
```

**两种模式对业务层的承诺一致**：`Transport` trait 提供 `incoming()` 接收消息、`send()` 发送消息。消息路由器、AI 客户端、会话管理完全不需要知道当前是哪种模式。

### 4.3 手机端架构 (iOS SwiftUI)

手机端作为"便携终端"和"手表代理"，需具备稳定的 WebSocket 连接能力（含断线重连）、二维码扫描配对能力，以及在不同连接模式间切换的能力。

**模块划分**：

| 模块 | 路径 | 职责 |
|------|------|------|
| **UI 层** | `iOSApp/Views/` | 聊天界面、设置页面、连接状态指示、二维码扫描 |
| **ViewModel** | `iOSApp/ViewModels/` | 会话状态管理、消息收发逻辑 (Combine) |
| **Transport 抽象** | `iOSApp/Network/Transport.swift` | 协议定义 + LanTransport / RelayTransport 实现 |
| **WebSocket 客户端** | `iOSApp/Network/WebSocketManager.swift` | URLSessionWebSocketTask 封装，含心跳、断线重连、指数退避 |
| **二维码扫描** | `iOSApp/Views/QRScannerView.swift` | AVFoundation 扫码解析配对信息（含模式识别） |
| **WCSession 管理器** | `iOSApp/WatchConnectivity/SessionManager.swift` | 与手表通信，转发消息 |
| **本地存储** | `iOSApp/Database/` | CoreData / SwiftData（会话缓存、离线消息） |
| **设备能力** | `iOSApp/Platform/` | 相机、相册、推送通知 |

#### 4.3.1 配对信息解析（识别连接模式）

```swift
// PairingInfo.swift
import Foundation

struct PairingInfo {
    enum Mode {
        case lan(ip: String, port: Int, session: String)
        case relay(url: String, room: String, key: String)
    }

    let mode: Mode

    /// 从二维码内容解析：aura://pair?mode=lan&ip=...&port=...&session=...
    /// 或：aura://pair?mode=relay&url=...&room=...&key=...
    static func parse(from qrString: String) -> PairingInfo? {
        guard let url = URL(string: qrString),
              url.scheme == "aura",
              url.host == "pair" else { return nil }

        let params = URLComponents(url: url, resolvingAgainstBaseURL: false)?
            .queryItems ?? []

        switch params.first(where: { $0.name == "mode" })?.value {
        case "lan":
            guard let ip = params.first(where: { $0.name == "ip" })?.value,
                  let portStr = params.first(where: { $0.name == "port" })?.value,
                  let port = Int(portStr),
                  let session = params.first(where: { $0.name == "session" })?.value
            else { return nil }
            return PairingInfo(mode: .lan(ip: ip, port: port, session: session))

        case "relay":
            guard let url = params.first(where: { $0.name == "url" })?.value,
                  let room = params.first(where: { $0.name == "room" })?.value,
                  let key = params.first(where: { $0.name == "key" })?.value
            else { return nil }
            return PairingInfo(mode: .relay(url: url, room: room, key: key))

        default:
            return nil
        }
    }
}
```

#### 4.3.2 WebSocket 客户端核心逻辑（含断线重连）

```swift
// WebSocketManager.swift
import Foundation
import Combine

class WebSocketManager: ObservableObject {
    private var webSocketTask: URLSessionWebSocketTask?
    private var reconnectTimer: Timer?
    private let maxReconnectDelay: TimeInterval = 30
    private var currentReconnectDelay: TimeInterval = 1

    @Published var isConnected = false
    @Published var latestMessage: String?

    private var host: String = ""
    private var port: Int = 0
    private var sessionId: String = ""

    init() {
        // App 回到前台时立即重连
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(appWillEnterForeground),
            name: UIApplication.willEnterForegroundNotification,
            object: nil
        )
    }

    func connect(host: String, port: Int, sessionId: String) {
        self.host = host
        self.port = port
        self.sessionId = sessionId

        let url = URL(string: "ws://\(host):\(port)/?session=\(sessionId)")!
        webSocketTask = URLSession.shared.webSocketTask(with: url)
        webSocketTask?.resume()
        isConnected = true
        currentReconnectDelay = 1  // 重置退避

        receiveMessage()
        startHeartbeat()
    }

    /// 心跳：每 30 秒发一次 ping
    private func startHeartbeat() {
        Timer.scheduledTimer(withTimeInterval: 30, repeats: true) { [weak self] _ in
            self?.sendPing()
        }
    }

    private func sendPing() {
        let ping: [String: Any] = [
            "type": "heartbeat",
            "timestamp": Date().timeIntervalSince1970
        ]
        send(json: ping)
    }

    func sendMessage(_ text: String) {
        let message = URLSessionWebSocketTask.Message.string(text)
        webSocketTask?.send(message) { error in
            if let error = error {
                print("发送失败: \(error)")
                self.handleDisconnect()
            }
        }
    }

    private func receiveMessage() {
        webSocketTask?.receive { [weak self] result in
            switch result {
            case .success(let message):
                switch message {
                case .string(let text):
                    self?.latestMessage = text
                case .data(let data):
                    // 处理二进制数据（图片等）
                    break
                @unknown default:
                    break
                }
                self?.receiveMessage() // 继续监听
            case .failure(let error):
                print("接收失败: \(error)")
                self?.handleDisconnect()
            }
        }
    }

    /// 断线自动重连，指数退避
    func handleDisconnect() {
        isConnected = false
        webSocketTask?.cancel()

        reconnectTimer?.invalidate()
        reconnectTimer = Timer.scheduledTimer(
            withTimeInterval: currentReconnectDelay,
            repeats: false
        ) { [weak self] _ in
            self?.retryConnect()
        }

        currentReconnectDelay = min(currentReconnectDelay * 2, maxReconnectDelay)
    }

    private func retryConnect() {
        guard !host.isEmpty else { return }
        connect(host: host, port: port, sessionId: sessionId)
    }

    /// App 回到前台时立即重连
    @objc func appWillEnterForeground() {
        if !isConnected {
            currentReconnectDelay = 1
            retryConnect()
        }
    }
}
```

#### 4.3.3 二维码扫描配对

```swift
// QRScannerView.swift
import SwiftUI
import AVFoundation

struct QRScannerView: UIViewControllerRepresentable {
    var onScan: (String) -> Void  // 返回 aura://pair?... 完整字符串

    func makeUIViewController(context: Context) -> QRScannerViewController {
        let vc = QRScannerViewController()
        vc.onScan = onScan
        return vc
    }

    func updateUIViewController(_ uiViewController: QRScannerViewController, context: Context) {}
}

class QRScannerViewController: UIViewController, AVCaptureMetadataOutputObjectsDelegate {
    var onScan: ((String) -> Void)?
    var captureSession: AVCaptureSession!

    override func viewDidLoad() {
        super.viewDidLoad()
        captureSession = AVCaptureSession()

        guard let videoCaptureDevice = AVCaptureDevice.default(for: .video) else { return }
        let videoInput = try? AVCaptureDeviceInput(device: videoCaptureDevice)

        if let videoInput = videoInput, captureSession.canAddInput(videoInput) {
            captureSession.addInput(videoInput)
        }

        let metadataOutput = AVCaptureMetadataOutput()
        if captureSession.canAddOutput(metadataOutput) {
            captureSession.addOutput(metadataOutput)
            metadataOutput.setMetadataObjectsDelegate(self, queue: DispatchQueue.main)
            metadataOutput.metadataObjectTypes = [.qr]
        }

        let previewLayer = AVCaptureVideoPreviewLayer(session: captureSession)
        previewLayer.frame = view.layer.bounds
        previewLayer.videoGravity = .resizeAspectFill
        view.layer.addSublayer(previewLayer)

        captureSession.startRunning()
    }

    func metadataOutput(_ output: AVCaptureMetadataOutput, didOutput metadataObjects: [AVMetadataObject], from connection: AVCaptureConnection) {
        if let metadataObject = metadataObjects.first as? AVMetadataMachineReadableCodeObject,
           let stringValue = metadataObject.stringValue {
            captureSession.stopRunning()
            onScan?(stringValue)
        }
    }
}
```

### 4.4 手表端架构 (watchOS SwiftUI)

手表端是"腕间指挥站"，核心是 AI 形象 + 实时语音对话 + 智能摘要显示。

**核心设计原则**：
- 不直接连桌面端：所有网络请求通过 iPhone App 代理转发（省电、稳定、安全）
- 不存储历史消息：只做实时收发中转，数据依赖 iPhone
- 视觉反馈优先于文本阅读：AI 形象动画是主要交互
- 智能摘要替代固定截断：AI 主动生成摘要，支持点击展开

> **手表端完全不感知连接模式** — 它只通过 WCSession 和 iPhone 通信，iPhone 用局域网还是中继服务器对它不可见。

**模块划分**：

| 模块 | 职责 |
|------|------|
| **AI 形象视图** | 动态动画（发光球体/粒子系统/抽象表情），表达 AI 状态 |
| **语音输入** | SFSpeechRecognizer 实时语音转文字 |
| **快捷指令** | 预设 3-5 个常用指令（一键发送） |
| **WCSession 通信** | 通过 WatchConnectivity 与配对的 iPhone App 通信 |
| **触觉反馈** | WKInterfaceDevice 震动提醒 |
| **摘要显示** | 显示 AI 生成的摘要，支持展开查看完整内容 |

#### 4.4.1 AI 形象状态机

```swift
// AIAvatarView.swift
import SwiftUI
import WatchKit

enum AIState {
    case idle       // 待命：柔和呼吸光晕
    case listening  // 倾听：波形脉动
    case thinking   // 思考：旋转粒子
    case speaking   // 回应：色彩流动
    case executing  // 执行中：进度环

    var colors: [Color] {
        switch self {
        case .idle: return [Color.blue.opacity(0.3), Color.purple.opacity(0.3)]
        case .listening: return [Color.yellow, Color.orange]
        case .thinking: return [Color.purple, Color.pink]
        case .speaking: return [Color.green, Color.teal]
        case .executing: return [Color.red, Color.orange]
        }
    }

    var isAnimating: Bool {
        self != .idle
    }
}

struct AIAvatarView: View {
    @State private var state: AIState = .idle
    @State private var glowAmount: Double = 0.5

    var body: some View {
        ZStack {
            // 动态发光球体
            Circle()
                .fill(
                    RadialGradient(
                        gradient: Gradient(colors: state.colors),
                        center: .center,
                        startRadius: 20,
                        endRadius: 60
                    )
                )
                .scaleEffect(glowAmount + 0.3)
                .animation(
                    state.isAnimating ?
                    Animation.easeInOut(duration: 1.5).repeatForever() :
                    .default,
                    value: glowAmount
                )

            // 状态指示点（"眼睛"）
            HStack(spacing: 12) {
                Circle()
                    .fill(state == .listening ? .yellow : .white)
                    .frame(width: 8, height: 8)
                    .offset(x: state == .listening ? -2 : 0)
                Circle()
                    .fill(state == .thinking ? .yellow : .white)
                    .frame(width: 8, height: 8)
                    .offset(x: state == .listening ? 2 : 0)
            }
        }
        .onChange(of: state) { _ in
            // 触发触觉反馈
            if state == .thinking || state == .executing {
                WKInterfaceDevice.current().play(.notification)
            }
        }
    }
}
```

#### 4.4.2 智能摘要显示（替代固定截断）

```swift
// WatchReplyView.swift
import SwiftUI

struct WatchReplyView: View {
    let message: PhoneToWatch
    @State private var isExpanded = false

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            // 摘要（优先显示，AI 主动生成）
            if let summary = message.summary {
                Text(summary)
                    .font(.system(size: 14, weight: .medium))
                    .foregroundColor(.primary)
                    .lineLimit(2)
            }

            // 展开按钮（有完整内容时显示）
            if message.content != nil && !isExpanded {
                Button("展开详情") {
                    isExpanded = true
                }
                .font(.system(size: 12))
                .foregroundColor(.accentColor)
            }

            // 完整内容（展开后显示）
            if isExpanded, let content = message.content {
                Text(content)
                    .font(.system(size: 12))
                    .foregroundColor(.gray)
                    .lineLimit(nil)
            }

            // 状态指示
            if message.status == .thinking {
                HStack {
                    ProgressView()
                        .scaleEffect(0.6)
                    Text("思考中...")
                        .font(.system(size: 11))
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding(.vertical, 4)
    }
}
```

### 4.5 通信链路详解

系统包含两条独立的通信链路，形成"两段式"架构：

```
┌─────────────────────────────────────────────────────────────────┐
│                    通信链路总览                                  │
│                                                                 │
│  段1: 桌面端 ↔ 手机端 (WebSocket)                                │
│       - 模式 A（局域网）：Mac 作为服务器监听，iPhone 直连         │
│       - 模式 B（中继）：两端都连用户的中继服务器，服务器盲转发    │
│       - 传输：JSON 消息、流式 AI 响应                            │
│       - 保护：心跳 30s / 超时 90s / 断线指数退避重连             │
│                                                                 │
│  段2: 手机端 ↔ 手表端 (WCSession，Apple 原生桥接)               │
│       - 手表通过 WCSession 发送消息给手机                       │
│       - 手机转发给桌面端；桌面端响应再通过手机推回手表            │
│       - 传输：轻量文本/状态指令/智能摘要                         │
│       - 与连接模式完全无关                                       │
└─────────────────────────────────────────────────────────────────┘
```

**为什么手表不直接连桌面端？**

| 原因 | 说明 |
|------|------|
| **省电** | WebSocket 长连接在手表上会严重消耗电池 |
| **稳定** | WCSession 由系统管理，后台保活机制更可靠 |
| **安全** | 手表通过 iPhone 的 trusted 链路访问，无需暴露桌面端端口给手表 |
| **简化** | 手表代码不需要实现 WebSocket 协议，只需与 iPhone 通信 |
| **模式无关** | iPhone 切换局域网/中继模式时手表代码零改动 |

**手机端作为"中转枢纽"的职责**：
1. 接收手表指令 → 转发给桌面端
2. 接收桌面端响应 → 推送给手表（实时流式转发）
3. 在手机端本地缓存消息（手表离线时暂存）
4. 管理断线重连，确保消息不丢失
5. 屏蔽连接模式差异（局域网或中继）对手表的影响

---

## 五、数据流与通信协议

### 5.1 端到端消息流（以手表语音指令为例）

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│ 手表端   │    │ iPhone  │    │  Mac    │    │ AI API  │
│ (watch) │    │ (手机)  │    │ (桌面端)│    │  API   │
└────┬────┘    └────┬────┘    └────┬────┘    └────┬────┘
     │              │              │              │
     │ 1.语音输入   │              │              │
     │ (AI形象:倾听)│              │              │
     │──────────────│              │              │
     │              │              │              │
     │ 2.WCSession │              │              │
     │ 转发文本    │              │              │
     │─────────────▶│              │              │
     │              │              │              │
     │              │ 3.WebSocket │              │
     │              │ 发送消息     │              │
     │              │ (含 seq 序号)│              │
     │              │ (模式 A/B 透明)             │
     │              │─────────────▶│              │
     │              │              │              │
     │              │              │ 4.调用API   │
     │              │              │─────────────▶│
     │              │              │              │
     │              │              │ 5.流式响应  │
     │              │              │◀─────────────│
     │              │              │              │
     │              │ 6.WebSocket  │              │
     │              │ 转发响应     │              │
     │              │ (含 seq 序号)│              │
     │              │◀─────────────│              │
     │              │              │              │
     │ 7.WCSession │              │              │
     │ 实时转发    │              │              │
     │ (含 summary)│              │              │
     │◀─────────────│              │              │
     │              │              │              │
     │ 8.显示摘要  │              │              │
     │ + 展开查看  │              │              │
     │ + 触觉反馈  │              │              │
     │ (AI形象:回应)│              │              │
```

### 5.2 WebSocket 通信协议（桌面端 ↔ 手机端）

**客户端 → 桌面端**：

```typescript
interface ClientToServer {
  type: 'message' | 'command' | 'join' | 'heartbeat' | 'file';
  sessionId: string;        // 会话 ID
  deviceId: string;         // 设备唯一标识
  deviceType: 'iphone' | 'watch';
  seq?: number;             // ← 可选：消息序号，用于去重
  payload: {
    text?: string;           // 用户消息
    command?: string;        // 快捷指令（预设）
    file?: {                 // 图片/文件（多模态）
      name: string;
      data: string;          // base64 编码
      type: string;
    };
  };
  timestamp: number;
}
```

**桌面端 → 客户端**：

```typescript
interface ServerToClient {
  type: 'message' | 'status' | 'sync' | 'error';
  sessionId: string;
  seq: number;              // ← 消息序号，用于去重和顺序保证
  from: string;              // 'ai' | 'system'
  payload: {
    content?: string;        // AI 响应内容（流式片段）
    summary?: string;        // ← AI 生成的摘要（手表端优先显示）
    fullHistory?: Message[]; // 全量历史（新设备加入时）
    status?: 'thinking' | 'executing' | 'done' | 'error';
    error?: string;
  };
  timestamp: number;
}
```

> **协议与连接模式无关** — 局域网和中继模式下，业务层消息体完全一致。中继服务器只做盲转发，不解析也不修改消息内容。

### 5.3 WCSession 通信协议（手表 ↔ 手机）

**手表 → 手机**：

```swift
struct WatchToPhone: Codable {
    type: 'speech' | 'quick_action' | 'ping'
    text: String?           // 语音转文字
    actionId: String?       // 快捷指令 ID
    sessionId: String       // 当前会话 ID
}
```

**手机 → 手表**：

```swift
struct PhoneToWatch: Codable {
    type: 'ai_reply' | 'status' | 'sync' | 'error'
    summary: String?        // ← AI 摘要（优先显示，30字以内）
    content: String?        // ← 完整回复（手表端可展开查看）
    fullReply: String?      // 完整回复（可选，用于在手机上查看）
    status: 'thinking' | 'speaking' | 'done'
    error: String?
}
```

### 5.4 AI 摘要生成策略

当检测到目标设备是手表时，桌面端通过 system prompt 要求 AI 同时输出摘要：

```rust
const WATCH_SYSTEM_PROMPT: &str = r#"
你是用户的 AI 助手。当用户通过 Apple Watch 与你交互时：
1. 先给出一句 30 字以内的核心结论作为 summary
2. 然后给出详细回复作为 content
3. 格式要求：{"summary": "核心结论", "content": "详细回复"}
4. 如果回复包含代码、路径、关键数据，确保 summary 中包含最关键的信息
"#;
```

---

## 六、数据模型

### 6.1 核心实体（桌面端 SQLite）

```sql
-- 会话表
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    title TEXT,
    created_at INTEGER,
    updated_at INTEGER,
    model TEXT DEFAULT 'deepseek-chat',
    system_prompt TEXT
);

-- 消息表
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id) ON DELETE CASCADE,
    role TEXT CHECK(role IN ('user', 'assistant', 'system')),
    content TEXT,
    summary TEXT,              -- ← AI 生成的摘要（手表端使用）
    seq INTEGER,               -- ← 消息序号，用于去重和顺序
    created_at INTEGER,
    attachments JSON           -- 文件引用 (file name, path, type)
);

-- 设备表（桌面端跟踪连接的设备）
CREATE TABLE connected_devices (
    id TEXT PRIMARY KEY,
    type TEXT CHECK(type IN ('iphone', 'watch')),
    session_id TEXT,
    last_seen INTEGER,
    ip_address TEXT,
    is_paired BOOLEAN DEFAULT FALSE  -- ← 是否已配对确认
);

-- 配对记录表（首次连接确认）
CREATE TABLE pairing_requests (
    id TEXT PRIMARY KEY,
    device_id TEXT,
    device_type TEXT,
    session_id TEXT,
    status TEXT CHECK(status IN ('pending', 'approved', 'rejected')),
    created_at INTEGER
);

-- 配置表
CREATE TABLE config (
    key TEXT PRIMARY KEY,
    value TEXT
);
-- 例如:
--   ('ai_base_url', 'https://api.deepseek.com/v1')   -- OpenAI 兼容协议 base_url
--   ('ai_api_key', 'sk-xxx')
--   ('ai_model', 'deepseek-chat')                    -- 模型名（gpt-4o / glm-4.6 / kimi-k2 等）
--   ('server_port', '8765')
--   ('connection_mode', 'lan')              -- 'lan' | 'relay'
--   ('relay_server_url', 'wss://user-vps.com/relay')
--   ('relay_room_id', 'auto-generated-uuid')
--   ('relay_secret_key', 'user-set-or-auto-generated')
```

### 6.2 手机端本地缓存（SwiftData）

```swift
@Model
class CachedMessage {
    var id: String
    var sessionId: String
    var role: String           // "user" / "assistant"
    var content: String
    var summary: String?       // ← 缓存摘要
    var seq: Int               // ← 消息序号
    var timestamp: Date
    var isFromWatch: Bool      // 标记是否来自手表
    var isSynced: Bool         // 是否已同步到桌面端
}
```

---

## 七、部署与运维

### 7.1 部署方式

| 组件 | 部署方式 | 说明 |
|------|---------|------|
| 桌面端 (Mac) | `.app` 安装包 | Tauri 打包，通过官网或 Homebrew 分发；支持菜单栏常驻；设置面板可选连接模式 |
| 手机端 (iOS) | App Store | 通过 TestFlight 内测后正式上架 |
| 手表端 (watchOS) | 随 iOS App 一起提交 | watchOS App 作为 iOS App 的 Extension |
| **中继服务器（可选）** | 用户自备 VPS + Docker | 用户在自己的 VPS 上跑开源镜像；仅在"自定义服务器"模式下使用 |

### 7.2 首次使用流程

1. 用户下载桌面端 App，在设置面板填入 AI 服务配置（Base URL / API Key / Model，可点选常用 Provider 快捷填入）
2. 在设置面板选择**连接模式**：
   - **局域网（默认）**：Mac 作为 WebSocket 服务器
   - **自定义服务器（可选）**：填入自部署的中继服务器地址（`wss://your-vps.com/relay`）
3. 点击"启动服务器"（局域网模式）或"连接服务器"（中继模式），应用缩至菜单栏后台运行
4. 点击"显示配对码"，Mac 屏幕显示二维码（含模式信息）
5. 手机端 App 扫描二维码，自动识别模式并解析连接参数
6. **局域网模式**首次连接需在桌面端点击"确认配对"（防未授权设备）；**中继模式**用户已通过 room/key 自授权，无需二次确认
7. 连接成功后，所有设备共享同一会话

### 7.3 局域网发现（Bonjour / mDNS + 二维码 Fallback）

**方案一：Bonjour 自动发现（首选，仅局域网模式）**

```rust
// 桌面端 (Rust) - 使用 mdns-sd 库
use mdns_sd::{ServiceDaemon, ServiceInfo};

fn publish_service() {
    let daemon = ServiceDaemon::new().unwrap();
    let service = ServiceInfo::new(
        "_aiassistant._tcp.local.",
        "My AI Assistant",
        "my-device.local.",
        8765,
        &["session=default"],
    );
    daemon.register(service).unwrap();
}
```

```swift
// 手机端 (Swift) - 使用 NWBrowser
import Network

let browser = NWBrowser(for: .bonjour(type: "_aiassistant._tcp", domain: "local."), using: .tcp)
browser.start(queue: .main)
```

**方案二：二维码配对（Bonjour 失效时 Fallback，两种模式通用）**

- Mac 端生成包含 `aura://pair?mode=lan&ip=...&port=...&session=xxx` 或 `aura://pair?mode=relay&url=...&room=...&key=...` 的二维码
- iPhone 相机扫码自动解析模式与参数，无需用户干预
- 无需手动输入 IP / 服务器地址

**方案三：手动输入（最终 Fallback）**

- 局域网模式：手动输入 IP + 端口 + Session ID
- 中继模式：手动输入服务器 URL + Room + Key

### 7.4 外网访问预留（用户自备 WebSocket 中继服务器）

不在第一阶段实现，但架构上预留接入点。与 Tailscale 方案相比，"用户自备中继"更符合"零云端依赖"精神 —— 中继是用户自己的，不是第三方 SaaS。

| 阶段 | 计划 |
|------|------|
| Phase 0-3 | 仅支持局域网直连 |
| Phase 4 | 实现 `RelayTransport`，用户可在 Mac 设置中切换到"自定义服务器"模式 |
| Phase 5+ | 开源极简 WebSocket 中继服务 Docker 镜像，用户一键部署到自己 VPS |

**中继服务器的最小职责**：

```typescript
// 伪代码：用户 VPS 上的中继服务器（< 200 行）
// 接受 WebSocket 连接，按 room 转发消息，不解析业务内容

wss.on('connection', (ws, req) => {
  const { room, key } = parseQuery(req);
  if (!validateKey(room, key)) return ws.close();

  ws.join(room);
  ws.on('message', (msg) => {
    // 广播给同 room 的其他客户端（盲转发）
    wss.to(room).except(ws).send(msg);
  });
});
```

**中继服务器不参与**：
- 业务消息解析
- AI 调用
- 持久化存储
- 认证（除 room/key 校验外）

**端到端加密策略（推荐）**：
- Mac 和 iPhone 在配对时协商共享密钥（基于二维码中的 session/key）
- 所有业务消息在 `Transport` 层加密后再发送
- 中继服务器只转发密文，即使被入侵也看不到明文
- 这是"零云端依赖"精神的真正实现：数据离开局域网但不可被第三方读取

**Phase 4+ 用户提供的中继实现选项**：

| 选项 | 部署方式 | 适合场景 |
|------|---------|---------|
| 官方开源 Docker 镜像 | `docker run -p 443:443 aura/relay` | 多数用户 |
| frp / ngrok 端口转发 | 把 Mac 的 8765 暴露到公网 | 已有 frp 基础设施的用户 |
| 自写 WebSocket 转发 | 用户自己实现 | 极客 |

### 7.5 安全考虑

| 风险 | 应对策略 |
|------|---------|
| 局域网内未授权设备连接 | 首次连接需在桌面端点击"确认配对"（类似 AirDrop）；中继模式靠 room/key 自授权 |
| WebSocket 传输明文 | 局域网模式 Phase 0-3 接受明文（局域网可信）；中继模式 Phase 4 起强制端到端加密（WSS + 业务层加密） |
| API Key 泄露 | 仅存储在桌面端 Keychain (macOS)；手机端不接触 Key；通用 OpenAI 协议意味着用户可接入任意 provider，Key 不会绑死单一服务 |
| 多用户误连 | 每个会话生成唯一 Session ID，二维码/手动输入均需匹配；中继模式 room 全局唯一 |
| 断网消息丢失 | 消息 seq 序号 + 客户端本地缓存 + 重连后同步 |
| 中继服务器被入侵 | 业务层端到端加密，中继只看到密文；room/key 可定期轮换 |

---

## 八、开发路线图

### Phase 0：技术验证（1 周）

| 任务 | 说明 | 验收标准 |
|------|------|---------|
| Tauri WebSocket 服务器 Demo | Rust 稳定运行 WebSocket 服务器，支持多客户端 | 能同时接受 3+ 连接 |
| iOS URLSession WebSocket 客户端 | 手机连接电脑并收发消息 | 能发送/接收文本消息 |
| 心跳 + 断线重连 Demo | 验证断网后自动恢复 | 断开 WiFi 10 秒后恢复，消息不丢失 |
| iOS 后台挂起恢复测试 | App 后台 5 分钟回前台 1 秒内重连 | 模拟系统挂起场景，恢复延迟 < 1s |
| WCSession 基础通信 | 手表 ↔ 手机消息转发 | 手表发送"Hello"，手机收到并显示 |
| Transport trait 抽象验证 | LanTransport 跑通，预留 RelayTransport 接口 | 切换 transport 实现不影响业务代码 |

**里程碑**：三端通信链路打通，断线重连测试通过，Transport 抽象层验证可行。

### Phase 1：桌面端核心（2-3 周）

| 任务 | 优先级 | 说明 |
|------|--------|------|
| Tauri 项目初始化 + UI | P0 | 聊天界面、设置面板（含连接模式开关）、菜单栏常驻 |
| AI 服务接入（通用 OpenAI 兼容） | P0 | 流式对话、多轮上下文、手表模式摘要生成；支持任意 base_url + model 组合 |
| WebSocket 服务器实现 (LanTransport) | P0 | 监听端口、连接管理（心跳/超时）、消息路由 |
| Transport trait 定义 | P0 | 抽象接口，LanTransport 实现 |
| 会话存储 | P0 | SQLite 保存历史和配置 |
| 二维码配对生成 | P1 | 生成含模式信息的配对二维码 |
| 文件拖入上传 | P1 | 拖入 txt/md/pdf 作为上下文 |
| 菜单栏常驻模式 | P1 | 关闭窗口后后台保持服务器运行 |

**里程碑**：桌面端可独立使用，且 WebSocket 服务器可被其他设备连接，断线重连稳定。

### Phase 2：手机端开发（2-3 周）

| 任务 | 优先级 | 说明 |
|------|--------|------|
| iOS SwiftUI 项目初始化 | P0 | 适配 iPhone + iPad |
| WebSocket 客户端 | P0 | 连接桌面端、消息收发、心跳、断线重连 |
| Transport 抽象 (Swift) | P0 | 协议定义 + LanTransport 实现，预留 RelayTransport |
| 聊天 UI | P0 | 消息列表、输入框、Markdown 渲染 |
| WCSession 管理器 | P0 | 与手表通信的桥梁 |
| 二维码扫描配对 | P1 | 扫码自动识别模式并连接 |
| 本地缓存 | P1 | SwiftData 缓存最近消息，支持离线查看 |
| 相机/相册集成 | P1 | 多模态图片输入 |

**里程碑**：手机端可连接桌面端，收发消息，断线自动恢复，并作为手表的通信代理。

### Phase 3：手表端开发（2-3 周）

| 任务 | 优先级 | 说明 |
|------|--------|------|
| watchOS SwiftUI 项目 | P0 | 与 iOS App 共享代码 |
| AI 形象动画 | P0 | 动态发光球体 + 5 状态状态机 |
| 语音识别集成 | P0 | SFSpeechRecognizer |
| WCSession 通信 | P0 | 与手机双向消息转发 |
| 智能摘要显示 | P0 | 显示 AI 摘要 + 点击展开完整内容 |
| 快捷指令 | P1 | 预设 3-5 个常用指令 |
| 触觉反馈 | P1 | 思考/回应时震动提醒 |

**里程碑**：手表可语音输入，AI 形象实时反馈，消息通过手机中转，摘要显示正常。

### Phase 4：外网访问与体验优化（持续迭代）

| 任务 | 优先级 | 说明 |
|------|--------|------|
| **RelayTransport 实现** | P0 | 桌面端 + iOS 端实现中继客户端模式，支持用户自备服务器 |
| **端到端加密** | P0 | 业务层加密，中继只看密文 |
| **中继服务器开源 Docker 镜像** | P1 | 用户一键部署到自己 VPS |
| Bonjour 局域网发现 | P1 | 手机自动发现电脑 |
| 多会话管理 | P1 | 新建/切换/删除会话 |
| 桌面端系统通知 | P1 | Mac 原生通知 |
| 手机推送通知 | P2 | 桌面端任务完成时远程提醒（中继模式必备） |
| 房间密钥轮换 | P2 | 中继模式定期换 room/key 增强安全 |

**里程碑**：用户出门用 5G 也能用手表遥控 Mac，中继服务器方案可用。

---

## 九、风险与应对

| 风险 | 影响 | 应对策略 |
|------|------|---------|
| 局域网 IP 变化 | 手机无法连接电脑 | Bonjour 自动发现 + 二维码配对 + 手动输入备选 |
| 电脑休眠/关机 | 服务不可用 | 菜单栏常驻提示状态；支持局域网唤醒 (WOL)（进阶） |
| iOS 后台断连 | 消息丢失/延迟 | 心跳 + 断线重连 + 指数退避 + App 前台立即重连 |
| AI API 限流或不可用 | AI 响应失败 | 实现重试 + 友好错误提示；通用 OpenAI 协议下用户可随时切换 base_url + model，不绑死单一 provider |
| watchOS 性能不足 | AI 形象动画卡顿 | 使用 Metal 渲染降级；动画帧率自适应 |
| 应用商店审核 | 手表 App 上架延迟 | 先通过 TestFlight 内测；准备审核说明 |
| 外网访问需求 | 用户抱怨"出门连不上" | Phase 4 实现 RelayTransport，架构已预留接口，用户自备 VPS 即可 |
| 中继服务器用户不会搭 | 普通用户用不上外网模式 | Phase 5+ 提供官方 Docker 镜像 + 文档，一键部署 |
| 中继服务器被入侵 | 消息内容泄露风险 | 业务层端到端加密，中继只转发密文 |

---

## 十、版本对比

### V1 vs V2 vs V2.1

| 维度 | V1 (云端中继版) | V2 (电脑直连版) | V2.1 (健壮性增强版) |
|------|-----------------|-----------------|---------------------|
| 移动端技术 | Flutter (跨平台) | SwiftUI (iOS + watchOS 原生) | SwiftUI (iOS + watchOS 原生) |
| 中继服务 | 云端 Node.js + Socket.io（官方运维） | 无，桌面端内置 WebSocket 服务器 | 无（局域网）；可选用户自备中继（外网） |
| 部署复杂度 | 需要部署云端服务 | 零云端依赖，开箱即用 | 零云端依赖，开箱即用 |
| 外网访问 | 支持（只要有网） | 仅限同一局域网 | 局域网 + 用户自备中继服务器（Phase 4+） |
| 数据安全 | 中继服务中转 | 数据不离开局域网 | 数据不离开局域网；外网模式下端到端加密 |
| 跨平台覆盖 | iOS + Android + Watch | 仅 Apple 生态 (Mac + iPhone + Watch) | 仅 Apple 生态 (Mac + iPhone + Watch) |
| 开发成本 | 三套技术栈 | 两套技术栈 (Tauri+Swift) | 两套技术栈 (Tauri+Swift) |
| Apple 生态整合 | 一般 | 深度整合（WCSession、SwiftUI 共享） | 深度整合 |
| **网络健壮性** | 依赖云端 | 基础 WebSocket | **心跳 + 重连 + 去重** |
| **手表交互** | 无 | 固定截断 30 字 | **AI 智能摘要 + 展开查看** |
| **设备配对** | 账号体系 | Bonjour + 手动输 IP | **二维码扫码（含模式识别）+ 手动输 IP** |
| **Mac 常驻** | 普通窗口 | 普通窗口 | **菜单栏常驻 + 后台服务** |
| **连接模式** | 仅云端 | 仅局域网 | **局域网 / 用户自备中继 双模式可切换** |

---

## 十一、总结

V2.1 架构的核心理念是：**"以 Mac 为中心，Apple 设备原生互联，网络层做厚，手表层做薄，桌面层做隐，外网做预留"**。

**优势**：
- 无云端依赖、数据本地化（局域网模式）
- Apple 生态深度整合
- 开发成本更低（iOS/watchOS 共享 Swift 代码）
- 网络层健壮（断线自动恢复，消息不丢失）
- 手表交互智能（摘要优先，详情可展开）
- 配对体验流畅（扫码即连，含模式识别）
- 外网访问用户自主可控（自备 VPS，端到端加密，不依赖第三方 SaaS）

**代价**：
- 仅限 Apple 生态
- 默认仅限局域网使用（外网模式需用户自备 VPS 并部署中继服务）

**下一步建议**：从 Phase 0 技术验证开始，重点验证**断线重连**和**心跳机制**，这是整个系统的稳定性基石；同时验证 **Transport trait 抽象**，确保后期切换到中继模式时业务代码零改动。
