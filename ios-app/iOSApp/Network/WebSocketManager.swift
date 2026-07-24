import Foundation
import Combine
import Network

/// 局域网模式下的 WebSocket 客户端
/// 心跳 30s / 超时 90s / 断线指数退避重连（1s→30s）/ App 前台立即重连
final class WebSocketManager: ObservableObject, Transport {
    @Published private(set) var status: TransportStatus = .stopped
    @Published var latestMessage: ServerMessage?

    private var webSocketTask: URLSessionWebSocketTask?
    private var reconnectTimer: Timer?
    private let maxReconnectDelay: TimeInterval = 30
    private var currentReconnectDelay: TimeInterval = 1
    private var heartbeatTimer: Timer?

    private var host: String = ""
    private var port: Int = 0
    private var sessionId: String = ""
    private let deviceId: String

    private let statusSubject = PassthroughSubject<TransportStatus, Never>()

    var statusPublisher: AnyPublisher<TransportStatus, Never> {
        statusSubject.eraseToAnyPublisher()
    }

    init(deviceId: String) {
        self.deviceId = deviceId
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(appWillEnterForeground),
            name: UIApplication.willEnterForegroundNotification,
            object: nil
        )
    }

    deinit {
        NotificationCenter.default.removeObserver(self)
        heartbeatTimer?.invalidate()
        reconnectTimer?.invalidate()
        webSocketTask?.cancel()
    }

    // MARK: - Transport

    func start() throws {
        guard !host.isEmpty else { throw TransportError.notConfigured }
        connect()
    }

    func stop() {
        heartbeatTimer?.invalidate()
        reconnectTimer?.invalidate()
        webSocketTask?.cancel(with: .goingAway, reason: nil)
        webSocketTask = nil
        setStatus(.stopped)
    }

    func send(_ text: String) throws {
        guard let task = webSocketTask else {
            throw TransportError.notConnected
        }
        task.send(.string(text)) { [weak self] error in
            if let error = error {
                print("发送失败: \(error)")
                self?.handleDisconnect()
            }
        }
    }

    // MARK: - LAN connect

    /// 用 PairingInfo 初始化
    func configure(with info: PairingInfo) {
        switch info.mode {
        case .lan(let ip, let port, let session):
            self.host = ip
            self.port = port
            self.sessionId = session
        case .relay:
            // Phase 4：中继模式接入
            setStatus(.error("Relay 模式 Phase 4 实现"))
        }
    }

    private func connect() {
        guard !host.isEmpty else { return }
        let url = URL(string: "ws://\(host):\(port)/?session=\(sessionId)")!
        webSocketTask = URLSession.shared.webSocketTask(with: url)
        webSocketTask?.resume()
        setStatus(.connected)
        currentReconnectDelay = 1

        // 先发 join 消息
        let join = ClientMessage.join(sessionId: sessionId, deviceId: deviceId)
        if let data = try? JSONEncoder().encode(join),
           let str = String(data: data, encoding: .utf8) {
            try? send(str)
        }

        receiveMessage()
        startHeartbeat()
    }

    // MARK: - Heartbeat

    private func startHeartbeat() {
        heartbeatTimer?.invalidate()
        heartbeatTimer = Timer.scheduledTimer(withTimeInterval: 30, repeats: true) { [weak self] _ in
            self?.sendPing()
        }
    }

    private func sendPing() {
        let ping = ClientMessage.heartbeat(deviceId: deviceId)
        if let data = try? JSONEncoder().encode(ping),
           let str = String(data: data, encoding: .utf8) {
            try? send(str)
        }
    }

    // MARK: - Receive

    private func receiveMessage() {
        webSocketTask?.receive { [weak self] result in
            switch result {
            case .success(let message):
                switch message {
                case .string(let text):
                    self?.handleText(text)
                case .data(let data):
                    if let text = String(data: data, encoding: .utf8) {
                        self?.handleText(text)
                    }
                @unknown default:
                    break
                }
                self?.receiveMessage()
            case .failure(let error):
                print("接收失败: \(error)")
                self?.handleDisconnect()
            }
        }
    }

    private func handleText(_ text: String) {
        guard let data = text.data(using: .utf8),
              let msg = try? JSONDecoder().decode(ServerMessage.self, from: data) else {
            print("无法解析: \(text)")
            return
        }
        DispatchQueue.main.async {
            self.latestMessage = msg
        }
    }

    // MARK: - Reconnect

    private func handleDisconnect() {
        setStatus(.connecting)
        webSocketTask?.cancel()
        webSocketTask = nil
        heartbeatTimer?.invalidate()

        reconnectTimer?.invalidate()
        reconnectTimer = Timer.scheduledTimer(
            withTimeInterval: currentReconnectDelay,
            repeats: false
        ) { [weak self] _ in
            self?.connect()
        }
        currentReconnectDelay = min(currentReconnectDelay * 2, maxReconnectDelay)
    }

    @objc private func appWillEnterForeground() {
        if status != .connected {
            currentReconnectDelay = 1
            connect()
        }
    }

    private func setStatus(_ s: TransportStatus) {
        DispatchQueue.main.async {
            self.status = s
            self.statusSubject.send(s)
        }
    }
}

enum TransportError: Error {
    case notConfigured
    case notConnected
}
