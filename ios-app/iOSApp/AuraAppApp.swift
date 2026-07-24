import SwiftUI

@main
struct AuraAppApp: App {
    @StateObject private var ws = WebSocketManager(deviceId: UIDevice.current.identifierForVendor?.uuidString ?? "unknown")

    var body: some Scene {
        WindowGroup {
            ContentView(ws: ws)
        }
    }
}

struct ContentView: View {
    @ObservedObject var ws: WebSocketManager

    var body: some View {
        NavigationView {
            VStack {
                switch ws.status {
                case .stopped:
                    Text("未连接").foregroundColor(.secondary)
                case .connecting:
                    ProgressView("重连中…")
                case .connected:
                    Label("已连接", systemImage: "checkmark.circle.fill")
                        .foregroundColor(.green)
                case .listening, .error:
                    EmptyView()
                case .error(let msg):
                    Text(msg).foregroundColor(.red).font(.caption)
                default:
                    EmptyView()
                }

                if let msg = ws.latestMessage,
                   let content = msg.payload?.content {
                    ScrollView {
                        Text(content)
                            .padding()
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }
            .navigationTitle("Aura")
            .toolbar {
                NavigationLink("配对", destination: PairingView(ws: ws))
            }
        }
    }
}

struct PairingView: View {
    @ObservedObject var ws: WebSocketManager
    @Environment(\.dismiss) var dismiss
    @State private var scanned: String?

    var body: some View {
        VStack {
            QRScannerView { result in
                scanned = result
                if let info = PairingInfo.parse(from: result) {
                    ws.configure(with: info)
                    try? ws.start()
                    dismiss()
                }
            }
            .frame(maxWidth: .infinity, maxHeight: 250)
            .cornerRadius(12)
            .padding()

            if let s = scanned {
                Text("扫描结果：\(s)").font(.caption).padding()
            }
        }
        .navigationTitle("扫码配对")
    }
}
