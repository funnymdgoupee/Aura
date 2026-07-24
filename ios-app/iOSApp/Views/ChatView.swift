import SwiftUI

/// 主聊天视图 — 消息列表 + 输入框
struct ChatView: View {
    @ObservedObject var session: SessionManager
    @ObservedObject var ws: WebSocketManager

    var body: some View {
        VStack(spacing: 0) {
            // 状态条
            statusBar

            Divider()

            // 消息列表
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 12) {
                        if session.messages.isEmpty {
                            emptyHint
                                .frame(maxWidth: .infinity)
                                .padding(.top, 60)
                        }
                        ForEach(session.messages) { m in
                            MessageBubble(message: m)
                                .id(m.id)
                        }
                    }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 12)
                }
                .onChange(of: session.messages.count) { _ in
                    if let last = session.messages.last {
                        withAnimation {
                            proxy.scrollTo(last.id, anchor: .bottom)
                        }
                    }
                }
            }

            Divider()

            // 输入栏
            composer
        }
    }

    private var statusBar: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(statusColor)
                .frame(width: 8, height: 8)
            Text(statusText)
                .font(.caption)
                .foregroundStyle(.secondary)
            Spacer()
            Button("清空") {
                session.clear()
            }
            .font(.caption)
            .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
    }

    private var statusColor: Color {
        switch ws.status {
        case .connected: return .green
        case .connecting: return .orange
        case .error: return .red
        default: return .gray
        }
    }

    private var statusText: String {
        switch ws.status {
        case .stopped: return "未连接"
        case .connecting: return "重连中…"
        case .connected: return "已连接"
        case .error(let msg): return "错误：\(msg)"
        case .listening: return ""
        }
    }

    private var emptyHint: some View {
        VStack(spacing: 8) {
            Image(systemName: "bubble.left.and.bubble.right")
                .font(.system(size: 40))
                .foregroundStyle(.tertiary)
            Text("发送一条消息开始对话")
                .font(.subheadline)
                .foregroundStyle(.secondary)
            Text("AI 回复会自动同步到 Mac 与 Apple Watch")
                .font(.caption)
                .foregroundStyle(.tertiary)
        }
    }

    private var composer: some View {
        HStack(spacing: 10) {
            TextField("输入消息", text: $session.input, axis: .vertical)
                .lineLimit(1...5)
                .padding(.horizontal, 14)
                .padding(.vertical, 10)
                .background(Color(.secondarySystemBackground))
                .clipShape(RoundedRectangle(cornerRadius: 18))

            Button(action: { session.send() }) {
                Image(systemName: "arrow.up")
                    .font(.system(size: 16, weight: .bold))
                    .foregroundStyle(.white)
                    .frame(width: 36, height: 36)
                    .background(session.input.isEmpty ? Color.gray : Color.accentColor)
                    .clipShape(Circle())
            }
            .disabled(session.input.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
    }
}

/// 单条消息气泡
struct MessageBubble: View {
    let message: ChatMessage
    @State private var cursorVisible = true

    private let cursorTimer = Timer.publish(every: 0.5, on: .main, in: .common).autoconnect()

    var body: some View {
        HStack {
            if message.role == .user {
                Spacer()
            }
            bubble
            if message.role != .user {
                Spacer()
            }
        }
    }

    @ViewBuilder
    private var bubble: some View {
        if message.thinking {
            HStack(spacing: 4) {
                ForEach(0..<3) { i in
                    Circle()
                        .fill(Color.secondary)
                        .frame(width: 6, height: 6)
                        .opacity(0.6)
                        .animation(
                            .easeInOut(duration: 0.6)
                                .repeatForever()
                                .delay(Double(i) * 0.2),
                            value: message.thinking
                        )
                }
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 12)
            .background(Color(.secondarySystemBackground))
            .clipShape(RoundedRectangle(cornerRadius: 16))
        } else if message.role == .user {
            Text(message.text)
                .foregroundStyle(.white)
                .padding(.horizontal, 14)
                .padding(.vertical, 10)
                .background(Color.accentColor)
                .clipShape(RoundedRectangle(cornerRadius: 16))
        } else if message.role == .system {
            Text(message.text)
                .font(.caption)
                .foregroundStyle(.red)
                .padding(.horizontal, 12)
                .padding(.vertical, 8)
                .background(Color.red.opacity(0.1))
                .clipShape(RoundedRectangle(cornerRadius: 12))
        } else {
            // AI — 流式时在末尾加闪烁光标
            VStack(alignment: .leading, spacing: 6) {
                if let summary = message.summary, !summary.isEmpty {
                    Text(summary)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .padding(.bottom, 4)
                        .overlay(Divider(), alignment: .bottom)
                }
                HStack(spacing: 1) {
                    Text(message.text)
                    if message.isStreaming {
                        Rectangle()
                            .fill(Color.primary)
                            .frame(width: 2, height: 14)
                            .opacity(cursorVisible ? 1 : 0)
                            .animation(.easeInOut(duration: 0.15), value: cursorVisible)
                    }
                }
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 10)
            .background(Color(.secondarySystemBackground))
            .clipShape(RoundedRectangle(cornerRadius: 16))
            .onReceive(cursorTimer) { _ in
                guard message.isStreaming else { return }
                cursorVisible.toggle()
            }
        }
    }
}
