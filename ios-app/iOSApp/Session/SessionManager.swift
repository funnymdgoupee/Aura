import Foundation
import Combine

/// 会话管理器 — WebSocketManager 与聊天 UI 之间的中间层
///
/// 职责：
/// 1. 持有当前会话消息列表（[ChatMessage]）
/// 2. 订阅 WebSocketManager 的 latestMessage，把 ServerMessage 翻译成 ChatMessage 追加到列表
/// 3. 提供 send(text) 方法：追加用户消息 → 编码 ClientMessage → 发到 ws
@MainActor
final class SessionManager: ObservableObject {
    @Published private(set) var messages: [ChatMessage] = []
    @Published var input: String = ""

    /// 当前会话 ID（扫码配对时拿到）
    private(set) var sessionId: String = "ios-default"

    /// 设备 ID（用 IDFV，简化处理）
    private let deviceId: String

    private let ws: WebSocketManager
    private var cancellables = Set<AnyCancellable>()

    init(ws: WebSocketManager, deviceId: String) {
        self.ws = ws
        self.deviceId = deviceId
        observeMessages()
    }

    // MARK: - Public

    /// 设置会话 ID（PairingInfo 解析后调）
    func configure(sessionId: String) {
        self.sessionId = sessionId
    }

    /// 发送一条消息
    func send() {
        let text = input.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return }

        // 1. 本地追加用户消息（即时反馈）
        messages.append(.user(text))

        // 2. 追加 thinking 占位
        messages.append(.thinking())

        // 3. 编码并发到 ws
        let msg = ClientMessage.message(sessionId: sessionId, deviceId: deviceId, text: text)
        guard let data = try? JSONEncoder().encode(msg),
              let str = String(data: data, encoding: .utf8) else {
            replaceLastThinking(with: .system("编码失败"))
            return
        }
        do {
            try ws.send(str)
            input = ""
        } catch {
            replaceLastThinking(with: .system("发送失败：\(error.localizedDescription)"))
        }
    }

    /// 清空当前会话
    func clear() {
        messages.removeAll()
    }

    // MARK: - Private

    private func observeMessages() {
        ws.$latestMessage
            .compactMap { $0 }
            .sink { [weak self] server in
                self?.handle(server)
            }
            .store(in: &cancellables)
    }

    private func handle(_ server: ServerMessage) {
        // 只处理 type == "message" 的消息
        guard server.type == "message" else { return }
        let payload = server.payload
        let content = payload?.content ?? ""
        let summary = payload?.summary
        let status = payload?.status

        if status == .streaming {
            // 增量追加到上一条 AI 流式消息
            if let last = messages.last, last.role == .ai, last.isStreaming {
                messages[messages.count - 1].text += content
            } else {
                // 移除 thinking 占位
                if let last = messages.last, last.thinking {
                    messages.removeLast()
                }
                messages.append(.aiStreaming(content))
            }
            return
        }

        if status == .done {
            // 替换上一条流式 AI 消息为最终内容
            if let last = messages.last, last.role == .ai, last.isStreaming {
                messages[messages.count - 1] = .ai(text: content, summary: summary, isError: false)
            } else {
                if let last = messages.last, last.thinking {
                    messages.removeLast()
                }
                messages.append(.ai(text: content, summary: summary, isError: false))
            }
            return
        }

        // error 或其他 — 系统消息
        let isError = status == .error || server.from == .system
        if let last = messages.last, last.thinking {
            messages[messages.count - 1] = .ai(text: content, summary: summary, isError: isError)
        } else {
            messages.append(.ai(text: content, summary: summary, isError: isError))
        }
    }

    private func replaceLastThinking(with msg: ChatMessage) {
        if let last = messages.last, last.thinking {
            messages[messages.count - 1] = msg
        } else {
            messages.append(msg)
        }
    }
}
