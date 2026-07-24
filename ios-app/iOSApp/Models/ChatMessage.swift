import Foundation

/// iPhone 端本地展示用的消息模型（与 Mac 端 ChatMessage 对应）
struct ChatMessage: Identifiable, Equatable {
    let id: UUID
    let role: Role
    var text: String
    var summary: String?
    var thinking: Bool
    var isStreaming: Bool
    var error: Bool
    let timestamp: Date

    enum Role: String {
        case user
        case ai
        case system
    }

    static func == (lhs: ChatMessage, rhs: ChatMessage) -> Bool {
        lhs.id == rhs.id
    }

    static func user(_ text: String) -> ChatMessage {
        ChatMessage(
            id: UUID(),
            role: .user,
            text: text,
            summary: nil,
            thinking: false,
            isStreaming: false,
            error: false,
            timestamp: Date()
        )
    }

    static func thinking() -> ChatMessage {
        ChatMessage(
            id: UUID(),
            role: .ai,
            text: "",
            summary: nil,
            thinking: true,
            isStreaming: false,
            error: false,
            timestamp: Date()
        )
    }

    static func aiStreaming(_ initialText: String) -> ChatMessage {
        ChatMessage(
            id: UUID(),
            role: .ai,
            text: initialText,
            summary: nil,
            thinking: false,
            isStreaming: true,
            error: false,
            timestamp: Date()
        )
    }

    static func ai(text: String, summary: String?, isError: Bool) -> ChatMessage {
        ChatMessage(
            id: UUID(),
            role: isError ? .system : .ai,
            text: text,
            summary: summary,
            thinking: false,
            isStreaming: false,
            error: isError,
            timestamp: Date()
        )
    }

    static func system(_ text: String) -> ChatMessage {
        ChatMessage(
            id: UUID(),
            role: .system,
            text: text,
            summary: nil,
            thinking: false,
            isStreaming: false,
            error: true,
            timestamp: Date()
        )
    }
}
