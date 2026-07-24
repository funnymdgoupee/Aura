import SwiftUI

@main
struct AuraAppApp: App {
    @StateObject private var ws: WebSocketManager
    @StateObject private var session: SessionManager

    init() {
        let deviceId = UIDevice.current.identifierForVendor?.uuidString ?? "ios-unknown"
        let ws = WebSocketManager(deviceId: deviceId)
        _ws = StateObject(wrappedValue: ws)
        _session = StateObject(wrappedValue: SessionManager(ws: ws, deviceId: deviceId))
    }

    var body: some Scene {
        WindowGroup {
            ContentView(ws: ws, session: session)
        }
    }
}
