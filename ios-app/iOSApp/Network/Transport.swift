import Foundation
import Combine

/// 传输层协议 — 屏蔽局域网直连与中继服务器两种模式差异
protocol Transport: AnyObject {
    var statusPublisher: AnyPublisher<TransportStatus, Never> { get }

    func start() throws
    func stop()
    func send(_ text: String) throws
}

enum TransportStatus: Equatable {
    case stopped
    case listening
    case connected
    case connecting
    case error(String)
}

/// 桌面端发来的消息（与 src-tauri/src/protocol.rs ServerToClient 一致）
struct ServerMessage: Codable {
    enum FromType: String, Codable { case ai, system }
    enum AiStatus: String, Codable { case thinking, executing, streaming, done, error }

    // 简化：仅解析 Phase 0 所需字段
    let type: String
    let sessionId: String?
    let seq: Int?
    let from: FromType?
    let payload: Payload?

    struct Payload: Codable {
        let content: String?
        let summary: String?
        let status: AiStatus?
        let error: String?
    }
}

/// 客户端发往桌面端的消息（与 src-tauri/src/protocol.rs ClientToServer 一致）
struct ClientMessage: Codable {
    let type: String      // "message" | "join" | "heartbeat"
    let sessionId: String
    let deviceId: String
    let deviceType: String  // "iphone" | "watch"
    let seq: Int?
    let payload: [String: AnyCodable]?
    let timestamp: Int64

    static func message(sessionId: String, deviceId: String, text: String) -> ClientMessage {
        ClientMessage(
            type: "message",
            sessionId: sessionId,
            deviceId: deviceId,
            deviceType: "iphone",
            seq: nil,
            payload: ["text": AnyCodable(text)],
            timestamp: Int64(Date().timeIntervalSince1970)
        )
    }

    static func join(sessionId: String, deviceId: String) -> ClientMessage {
        ClientMessage(
            type: "join",
            sessionId: sessionId,
            deviceId: deviceId,
            deviceType: "iphone",
            seq: nil,
            payload: nil,
            timestamp: Int64(Date().timeIntervalSince1970)
        )
    }

    static func heartbeat(deviceId: String) -> ClientMessage {
        ClientMessage(
            type: "heartbeat",
            sessionId: "",
            deviceId: deviceId,
            deviceType: "iphone",
            seq: nil,
            payload: nil,
            timestamp: Int64(Date().timeIntervalSince1970)
        )
    }
}

/// JSON Any 包装（用于 payload 这种动态字段）
enum AnyCodable: Codable {
    case string(String)
    case int(Int)
    case double(Double)
    case bool(Bool)

    init(from decoder: Decoder) throws {
        let c = try decoder.singleValueContainer()
        if let v = try? c.decode(String.self) { self = .string(v) }
        else if let v = try? c.decode(Int.self) { self = .int(v) }
        else if let v = try? c.decode(Double.self) { self = .double(v) }
        else if let v = try? c.decode(Bool.self) { self = .bool(v) }
        else { self = .string("") }
    }

    func encode(to encoder: Encoder) throws {
        var c = encoder.singleValueContainer()
        switch self {
        case .string(let v): try c.encode(v)
        case .int(let v): try c.encode(v)
        case .double(let v): try c.encode(v)
        case .bool(let v): try c.encode(v)
        }
    }
}
