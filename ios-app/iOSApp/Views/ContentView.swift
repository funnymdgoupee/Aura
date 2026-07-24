import SwiftUI

/// 入口视图 — 根据连接状态切换"扫码配对"或"聊天"
struct ContentView: View {
    @ObservedObject var ws: WebSocketManager
    @ObservedObject var session: SessionManager

    var body: some View {
        NavigationStack {
            if ws.status == .connected || ws.status == .connecting {
                ChatView(session: session, ws: ws)
                    .navigationTitle("Aura")
                    .navigationBarTitleDisplayMode(.inline)
                    .toolbar {
                        NavigationLink {
                            PairingView(ws: ws, session: session)
                        } label: {
                            Image(systemName: "qrcode.viewfinder")
                        }
                    }
            } else {
                PairingView(ws: ws, session: session)
            }
        }
    }
}

/// 扫码配对视图
struct PairingView: View {
    @ObservedObject var ws: WebSocketManager
    @ObservedObject var session: SessionManager
    @Environment(\.dismiss) var dismiss
    @State private var scanned: String?
    @State private var parseError: String?

    var body: some View {
        VStack(spacing: 16) {
            QRScannerView { result in
                guard scanned == nil else { return }
                scanned = result
                if let info = PairingInfo.parse(from: result) {
                    switch info.mode {
                    case .lan(let ip, let port, let sid):
                        session.configure(sessionId: sid)
                        ws.configure(with: info)
                        try? ws.start()
                        dismiss()
                    case .relay:
                        parseError = "中继模式 Phase 4 实现"
                    }
                } else {
                    parseError = "二维码格式无效：\(result)"
                }
            }
            .frame(maxWidth: .infinity, maxHeight: 280)
            .clipShape(RoundedRectangle(cornerRadius: 16))
            .padding(.horizontal, 24)

            VStack(spacing: 6) {
                Text("扫描 Mac 端的配对二维码")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                Text("在 Mac App 中点击「配对二维码」生成")
                    .font(.caption)
                    .foregroundStyle(.tertiary)
            }

            if let parseError {
                Text(parseError)
                    .font(.caption)
                    .foregroundStyle(.red)
                    .padding(.horizontal)
            }
        }
        .padding(.vertical, 32)
        .navigationTitle("扫码配对")
    }
}
