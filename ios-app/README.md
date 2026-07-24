# iOS App（Phase 2）

这部分代码需要在 **macOS + Xcode** 环境下集成和编译。Windows 上无法构建 iOS App。

## 在 Mac 上的开发流程

1. 打开 Xcode → New Project → App
2. Product Name 填 `AuraApp`
3. Interface 选 SwiftUI
4. Language 选 Swift
5. 保存到 `F:\Aura\ios-app\AuraApp.xcodeproj`（或者你 clone 到 Mac 后的路径）
6. 把本目录下的源文件拖入 Xcode 工程的 `AuraApp` 组（保持"Create groups"）
7. 在 Signing & Capabilities 里勾选你的 Apple ID 团队
8. 接入 watchOS App Target（Phase 3）时，File → New → Target → watchOS App

## 文件清单

| 文件 | 职责 |
|------|------|
| `iOSApp/AuraAppApp.swift` | `@main` App 入口，注入 `WebSocketManager` + `SessionManager` |
| `iOSApp/Models/ChatMessage.swift` | 本地消息模型（user / ai / system / thinking 四态） |
| `iOSApp/Session/SessionManager.swift` | 会话管理器，订阅 ws 消息流，提供 send/clear |
| `iOSApp/Network/PairingInfo.swift` | 解析二维码 `aura://pair?...`，识别 lan/relay 模式 |
| `iOSApp/Network/Transport.swift` | Transport 协议 + ServerMessage/ClientMessage 编解码 |
| `iOSApp/Network/WebSocketManager.swift` | URLSessionWebSocketTask 封装，含心跳/断线重连 |
| `iOSApp/Views/ContentView.swift` | 入口视图，根据连接状态切换配对/聊天 |
| `iOSApp/Views/ChatView.swift` | 聊天视图：消息列表 + thinking 动画 + 输入栏 |
| `iOSApp/Views/QRScannerView.swift` | AVFoundation 扫码 |
| `iOSApp/WatchConnectivity/SessionManager.swift` | Phase 3 补齐，与手表通信 |
| `iOSApp/Database/CachedMessage.swift` | Phase 2 补齐，SwiftData 本地缓存 |

## Info.plist 权限说明

需要在 Info.plist 加：
- `NSCameraUsageDescription` — 用于扫码配对
- `NSLocalNetworkUsageDescription` — 连接局域网内 Mac
- `NSMicrophoneUsageDescription` — 语音输入（Phase 2 多模态）

## 当前完成度

- ✅ PairingInfo.swift（二维码解析，支持 lan/relay 双模式）
- ✅ Transport.swift（协议定义 + ServerMessage/ClientMessage 编解码）
- ✅ WebSocketManager.swift（心跳 + 断线重连 + 指数退避）
- ✅ ChatMessage.swift（本地消息模型，user/ai/system/thinking 四态）
- ✅ SessionManager.swift（订阅 ws，翻译为 [ChatMessage]，提供 send）
- ✅ ChatView.swift（消息气泡 + thinking 动画 + 输入栏 + 状态条）
- ✅ ContentView.swift（入口：未连接→扫码，已连接→ChatView）
- ✅ QRScannerView.swift（AVFoundation 扫码，摄像头预览 + QR 识别）
- ⏳ SwiftData 本地缓存（Phase 2）
- ⏳ watchOS Target + WatchConnectivity（Phase 3）

## 最小可用流程

1. Mac 端启动服务（局域网模式）
2. Mac 端点击「配对二维码」生成
3. iPhone 在 Aura App 里扫码
4. `PairingInfo` 解析出 IP/Port/SessionId
5. `WebSocketManager` 建立 ws 连接，发 `join`
6. iPhone 在输入栏发消息 → `SessionManager.send()` → ws
7. Mac router 收到 → 调 AI → 广播回来 → iPhone `SessionManager` 追加 AI 消息
