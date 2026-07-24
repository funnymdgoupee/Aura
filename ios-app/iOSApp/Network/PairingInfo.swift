import Foundation

/// 二维码配对信息解析
/// 支持 aura://pair?mode=lan&... 和 aura://pair?mode=relay&...
struct PairingInfo: Equatable {
    enum Mode: Equatable {
        case lan(ip: String, port: Int, session: String)
        case relay(url: String, room: String, key: String)
    }

    let mode: Mode

    static func parse(from qrString: String) -> PairingInfo? {
        guard let comps = URLComponents(string: qrString),
              comps.scheme == "aura",
              comps.host == "pair" else { return nil }

        let items = comps.queryItems ?? []
        func value(_ key: String) -> String? {
            items.first(where: { $0.name == key })?.value
        }

        switch value("mode") {
        case "lan":
            guard let ip = value("ip"),
                  let portStr = value("port"),
                  let port = Int(portStr),
                  let session = value("session") else { return nil }
            return PairingInfo(mode: .lan(ip: ip, port: port, session: session))

        case "relay":
            guard let url = value("url"),
                  let room = value("room"),
                  let key = value("key") else { return nil }
            return PairingInfo(mode: .relay(url: url, room: room, key: key))

        default:
            return nil
        }
    }
}
