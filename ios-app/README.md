# iOS App（Phase 2）

这部分代码需要在 **macOS + Xcode** 环境下集成和编译。Windows 上无法构建 iOS App。

## 在 Mac 上的开发流程

1. 打开 Xcode → New Project → App
2. Product Name 填 `AuraApp`
3. Interface 选 SwiftUI
4. Language 选 Swift
5. 保存到 `F:\Aura\ios-app\AuraApp.xcodeproj`（或者你 clone 到 Mac 后的路径）
6. 把本目录下的源文件拖入 Xcode 工程的 `AuraApp` 组
7. 在 Signing & Capabilities 里勾选你的 Apple ID 团队
8. 接入 watchOS App Target（Phase 3）时，File → New → Target → watchOS App

## 文件清单

| 文件 | 职责 |
|------|------|
| `iOSApp/Network/PairingInfo.swift` | 解析二维码 `aura://pair?...`，识别 lan/relay 模式 |
| `iOSApp/Network/Transport.swift` | Transport 协议 + LanTransport / RelayTransport 实现 |
| `iOSApp/Network/WebSocketManager.swift` | URLSessionWebSocketTask 封装，含心跳/断线重连 |
| `iOSApp/Views/QRScannerView.swift` | AVFoundation 扫码 |
| `iOSApp/Views/ChatView.swift` | Phase 2 补齐 |
| `iOSApp/WatchConnectivity/SessionManager.swift` | Phase 3 补齐，与手表通信 |
| `iOSApp/Database/CachedMessage.swift` | Phase 2 补齐，SwiftData 本地缓存 |
| `iOSApp/AuraAppApp.swift` | App 入口 |

## Info.plist 权限说明

需要在 Info.plist 加：
- `NSCameraUsageDescription` — 用于扫码配对
- `NSMicrophoneUsageDescription` — 语音输入（Phase 2 多模态）
- `NSLocalNetworkUsageDescription` — 连接局域网内 Mac

## 当前完成度

- ✅ PairingInfo.swift（二维码解析，支持 lan/relay 双模式）
- ✅ Transport.swift（协议定义 + LanTransport 实现）
- ✅ WebSocketManager.swift（心跳 + 断线重连 + 指数退避）
- ⏳ QRScannerView.swift（Phase 2）
- ⏳ ChatView 等其他（Phase 2）
