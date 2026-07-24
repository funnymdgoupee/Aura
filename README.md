# Aura Assistant

跨平台通用 AI 助手 — 以 Mac 为中心，Apple 设备原生互联。

详见架构设计文档：[AI_Assistant_Architecture_V2.1.md](./AI_Assistant_Architecture_V2.1.md)

## 当前状态（Phase 0 骨架）

| 模块 | 状态 |
|------|------|
| Rust 后端（Tauri 2） | ✅ 代码已写，未编译验证（需 Mac 环境） |
| Transport 抽象层 | ✅ trait + LanTransport 实现，RelayTransport 骨架 |
| WebSocket 服务器 | ✅ 心跳 30s / 超时 90s / 消息路由（echo） |
| 连接管理器 | ✅ 设备注册、心跳检测、超时清理、广播 |
| 二维码配对 | ✅ 生成 `aura://pair?mode=lan&...` / `mode=relay&...` |
| SQLite 存储 | ✅ schema + config CRUD（sessions/messages 等待 Phase 1） |
| AI 客户端 | ⏳ 通用 OpenAI 兼容客户端已写（流式 SSE + 非流式），Phase 1 接入路由器 |
| 菜单栏常驻 | ✅ Tauri 2 TrayIcon，关闭窗口改为隐藏 |
| React 前端 | ✅ 设置面板、连接模式开关、二维码预览、调试广播 |
| iOS App | ⏳ PairingInfo + Transport + WebSocketManager + 扫码 + App 入口，待 Mac 上 Xcode 集成 |
| watchOS App | ⏳ Phase 3 |

## 项目结构

```
F:\Aura\
├── AI_Assistant_Architecture_V2.1.md   # 架构设计文档
├── README.md                             # 本文件
├── .gitignore
├── package.json                          # Tauri CLI 入口
├── src-frontend/                         # React + Vite 前端
│   ├── package.json
│   ├── vite.config.ts
│   ├── tsconfig.json
│   ├── index.html
│   └── src/
│       ├── main.tsx
│       ├── App.tsx                       # 设置面板 + 调试 UI
│       └── styles.css
├── src-tauri/                            # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   └── src/
│       ├── main.rs                       # 入口
│       ├── lib.rs                        # 应用初始化、Tauri 命令注册
│       ├── config.rs                     # AppConfig + ConnectionMode
│       ├── protocol.rs                   # ClientToServer / ServerToClient 消息类型
│       ├── commands.rs                   # 暴露给前端的 Tauri 命令
│       ├── pairing.rs                    # 二维码生成
│       ├── tray.rs                       # 菜单栏常驻
│       ├── ai/mod.rs                     # 通用 OpenAI 兼容 AI 客户端（流式 SSE + 非流式 + watch 模式 summary）
│       ├── db/mod.rs                     # SQLite 存储
│       ├── network/
│       │   ├── mod.rs                    # Transport trait + TransportHandle
│       │   ├── lan.rs                    # LanTransport（WebSocket 服务器）
│       │   └── relay.rs                  # RelayTransport（Phase 4 骨架）
│       └── server/
│           ├── mod.rs
│           ├── connections.rs            # ConnectionManager（心跳/超时/广播）
│           └── router.rs                 # 消息路由（Phase 0 echo）
└── ios-app/                              # iOS App（Mac 上 Xcode 编译）
    ├── README.md                         # Xcode 集成说明
    └── iOSApp/
        ├── AuraAppApp.swift              # App 入口
        ├── Network/
        │   ├── PairingInfo.swift          # 二维码解析
        │   ├── Transport.swift            # Transport 协议
        │   └── WebSocketManager.swift     # 含心跳/断线重连
        └── Views/
            └── QRScannerView.swift       # AVFoundation 扫码
```

## 在 Mac 上的开发流程

### 1. 准备工具链

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 Node（推荐用 nvm 或 brew）
brew install node

# 安装 Tauri 2 前端依赖
cd /path/to/Aura
npm install
```

### 2. 添加 Tauri 图标

`tauri.conf.json` 引用了 `src-tauri/icons/` 下的图标文件，第一次运行前需要生成：

```bash
# 准备一张 1024x1024 的 PNG 放到任意位置，然后：
npm run tauri icon path/to/source.png
```

这会自动生成所有需要的尺寸（包括 `.icns` 和 `.ico`）。

### 3. 启动开发模式

```bash
npm run dev
```

这会：
- 启动 Vite dev server（前端热更新）
- 编译 Rust 后端
- 弹出 Tauri 窗口

第一次编译 Rust 较慢（5-15 分钟），之后增量编译很快。

### 4. Phase 0 验收

启动后界面显示"启动服务"按钮 → 点击 → 状态变成 `lan · listening` → 点"配对二维码"显示二维码。

用任何 WebSocket 客户端连 `ws://<Mac-IP>:8765/?session=test` 验证：
- 发送 `{"type":"join","session_id":"test","device_id":"dev1","device_type":"iphone","timestamp":0}` → 服务端日志显示设备加入
- 发送 `{"type":"heartbeat","device_id":"dev1","timestamp":0}` → 心跳更新
- 发送 `{"type":"message","session_id":"test","device_id":"dev1","device_type":"iphone","seq":1,"payload":{"text":"hi"},"timestamp":0}` → 收到 echo 回复 `{"type":"message","session_id":"test","seq":1,"from":"ai","payload":{"content":"[echo] hi","summary":"hi","status":"done"}}`

### 5. iOS App（需要 Xcode）

参考 [ios-app/README.md](./ios-app/README.md)。

## 下一步任务（Phase 1）

按 V2.1 文档第八节路线图：

- [ ] AI 调用接入路由器（`ai/mod.rs` 已写完流式 + 非流式，待 router 调用）
- [ ] 会话管理（`session/` 模块，CRUD + 多轮上下文）
- [ ] 手表模式摘要生成（system prompt + JSON 解析）
- [ ] 消息路由器接入真实 AI 客户端（替换 `server/router.rs` 的 echo）
- [ ] 文件拖入上传（拖入 txt/md/pdf 作为上下文）
- [ ] iOS App Xcode 工程化（Phase 2）

## 已知问题

- 代码在 Windows 上写出，未通过 `cargo check`，可能有少量编译错误需要在 Mac 上修正
- `mdns-sd` 依赖的 Bonjour 发布函数尚未在 `commands.rs` 调用（Phase 4 任务）
- Tauri 2 TrayIcon API 在不同平台行为差异需要真机验证
