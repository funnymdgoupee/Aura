import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

type ConnectionMode = "lan" | "relay";

interface AppConfig {
  deepseek_api_key: string;
  server_port: number;
  connection_mode: ConnectionMode;
  relay_server_url: string;
  relay_room_id: string;
  relay_secret_key: string;
}

interface StatusResponse {
  mode: string;
  status: string;
  port: number | null;
  server_url: string | null;
  error: string | null;
}

interface PairingResult {
  qr_data_url: string;
  raw_url: string;
}

interface ChatMessage {
  role: "user" | "ai";
  text: string;
}

export default function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [qr, setQr] = useState<PairingResult | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");

  const refreshStatus = useCallback(async () => {
    try {
      const s = await invoke<StatusResponse>("get_transport_status");
      setStatus(s);
    } catch (e) {
      console.error("获取状态失败", e);
    }
  }, []);

  const refreshConfig = useCallback(async () => {
    try {
      const c = await invoke<AppConfig>("get_config");
      setConfig(c);
    } catch (e) {
      console.error("获取配置失败", e);
    }
  }, []);

  useEffect(() => {
    refreshConfig();
    refreshStatus();
    const timer = setInterval(refreshStatus, 3000);
    return () => clearInterval(timer);
  }, [refreshConfig, refreshStatus]);

  const handleStart = async () => {
    try {
      await invoke("start_with_mode");
      await refreshStatus();
    } catch (e) {
      alert("启动失败: " + e);
    }
  };

  const handleStop = async () => {
    try {
      await invoke("stop_transport");
      await refreshStatus();
    } catch (e) {
      alert("停止失败: " + e);
    }
  };

  const handleGenerateQr = async () => {
    try {
      const r = await invoke<PairingResult>("generate_pairing_qr");
      setQr(r);
    } catch (e) {
      alert("生成二维码失败: " + e);
    }
  };

  const handleSwitchMode = async (mode: ConnectionMode) => {
    try {
      await invoke("switch_connection_mode", { mode });
      await refreshConfig();
      await refreshStatus();
    } catch (e) {
      alert("切换模式失败: " + e);
    }
  };

  const handleSaveConfig = async () => {
    if (!config) return;
    try {
      await invoke("save_config", { config });
      alert("已保存");
    } catch (e) {
      alert("保存失败: " + e);
    }
  };

  const handleSend = async () => {
    if (!input.trim()) return;
    const text = input;
    setMessages((m) => [...m, { role: "user", text }]);
    setInput("");
    try {
      await invoke("send_test_message", { text });
      setMessages((m) => [...m, { role: "ai", text: `[echo] ${text}` }]);
    } catch (e) {
      setMessages((m) => [...m, { role: "ai", text: `发送失败: ${e}` }]);
    }
  };

  if (!config) {
    return <div className="app"><div className="header"><h1>加载中...</h1></div></div>;
  }

  const running = status?.status === "listening" || status?.status === "connected";

  return (
    <div className="app">
      <header className="header">
        <h1>Aura Assistant</h1>
        <span className={`status ${running ? "running" : ""} ${status?.status === "error" ? "error" : ""}`}>
          {status ? `${status.mode} · ${status.status}` : "—"}
        </span>
        {running ? (
          <button className="btn danger" onClick={handleStop}>停止</button>
        ) : (
          <button className="btn" onClick={handleStart}>启动服务</button>
        )}
        <button className="btn secondary" onClick={handleGenerateQr}>配对二维码</button>
      </header>

      <div className="main">
        <aside className="sidebar">
          <h2 className="section-title">连接模式</h2>
          <div className="field">
            <select
              value={config.connection_mode}
              onChange={(e) => handleSwitchMode(e.target.value as ConnectionMode)}
            >
              <option value="lan">局域网（Mac 作为服务器）</option>
              <option value="relay">自定义服务器（中继，Phase 4）</option>
            </select>
          </div>

          {config.connection_mode === "lan" ? (
            <div className="field">
              <label>监听端口</label>
              <input
                type="number"
                value={config.server_port}
                onChange={(e) =>
                  setConfig({ ...config, server_port: parseInt(e.target.value) || 8765 })
                }
              />
            </div>
          ) : (
            <>
              <div className="field">
                <label>中继服务器地址</label>
                <input
                  type="text"
                  placeholder="wss://your-vps.com/relay"
                  value={config.relay_server_url}
                  onChange={(e) =>
                    setConfig({ ...config, relay_server_url: e.target.value })
                  }
                />
              </div>
              <div className="field">
                <label>Room ID（自动生成）</label>
                <input
                  type="text"
                  value={config.relay_room_id}
                  onChange={(e) =>
                    setConfig({ ...config, relay_room_id: e.target.value })
                  }
                />
              </div>
              <div className="field">
                <label>Secret Key</label>
                <input
                  type="password"
                  value={config.relay_secret_key}
                  onChange={(e) =>
                    setConfig({ ...config, relay_secret_key: e.target.value })
                  }
                />
              </div>
              <p style={{ fontSize: 11, color: "#8e8e93", marginTop: 0 }}>
                中继模式 Phase 4 实现，当前仅作配置占位。
              </p>
            </>
          )}

          <h2 className="section-title" style={{ marginTop: 20 }}>DeepSeek API</h2>
          <div className="field">
            <label>API Key</label>
            <input
              type="password"
              placeholder="sk-xxx"
              value={config.deepseek_api_key}
              onChange={(e) =>
                setConfig({ ...config, deepseek_api_key: e.target.value })
              }
            />
          </div>

          <button className="btn secondary" onClick={handleSaveConfig} style={{ width: "100%" }}>
            保存配置
          </button>

          {qr && (
            <>
              <h2 className="section-title" style={{ marginTop: 20 }}>配对二维码</h2>
              <img src={qr.qr_data_url} alt="pairing qr" className="qr-preview" />
              <p style={{ fontSize: 11, color: "#8e8e93", wordBreak: "break-all" }}>
                {qr.raw_url}
              </p>
            </>
          )}
        </aside>

        <main className="chat">
          {messages.length === 0 && (
            <div style={{ color: "#8e8e93", fontSize: 13, textAlign: "center", marginTop: 40 }}>
              启动服务后，用手机扫码连接，或在下方输入测试消息广播给已连接的设备。
            </div>
          )}
          {messages.map((m, i) => (
            <div key={i} className={`msg ${m.role}`}>{m.text}</div>
          ))}
        </main>
      </div>

      <div className="composer">
        <input
          type="text"
          placeholder="输入测试消息（广播给所有已连接设备）"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleSend()}
        />
        <button className="btn" onClick={handleSend}>发送</button>
      </div>
    </div>
  );
}
