import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

type ConnectionMode = "lan" | "relay";

interface AppConfig {
  ai_base_url: string;
  ai_api_key: string;
  ai_model: string;
  server_port: number;
  connection_mode: ConnectionMode;
  relay_server_url: string;
  relay_room_id: string;
  relay_secret_key: string;
}

interface ProviderPreset {
  label: string;
  base_url: string;
  model: string;
  doc_url?: string;
}

const PROVIDER_PRESETS: ProviderPreset[] = [
  { label: "DeepSeek", base_url: "https://api.deepseek.com/v1", model: "deepseek-chat" },
  { label: "OpenAI", base_url: "https://api.openai.com/v1", model: "gpt-4o" },
  { label: "Claude (兼容代理)", base_url: "https://api.anthropic.com/v1/openai", model: "claude-sonnet-4-6" },
  { label: "Gemini", base_url: "https://generativelanguage.googleapis.com/v1beta/openai", model: "gemini-2.5-flash" },
  { label: "智谱 GLM", base_url: "https://open.bigmodel.cn/api/paas/v4", model: "glm-4.6" },
  { label: "通义 Qwen", base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1", model: "qwen3-coder-plus" },
  { label: "Kimi", base_url: "https://api.moonshot.cn/v1", model: "kimi-k2" },
  { label: "豆包", base_url: "https://ark.cn-beijing.volces.com/api/v3", model: "doubao-1-5-pro" },
  { label: "Ollama (本地)", base_url: "http://localhost:11434/v1", model: "qwen2.5" },
];

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
  role: "user" | "ai" | "system";
  text: string;
  summary?: string;
  thinking?: boolean;
  error?: boolean;
}

interface AiStatusEvent {
  session_id: string;
  status: "thinking" | "executing" | "done" | "error";
  error?: string;
}

interface AiMessageEvent {
  session_id: string;
  seq: number;
  from: "ai" | "system";
  payload: {
    content?: string;
    summary?: string;
    status?: "thinking" | "executing" | "done" | "error";
    error?: string;
  };
  timestamp: number;
}

interface AiErrorEvent {
  session_id: string;
  code: string;
  message: string;
  timestamp: number;
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

  // 监听 router 发来的 AI 消息 / 状态 / 错误
  useEffect(() => {
    let unlistenMsg: UnlistenFn | undefined;
    let unlistenStatus: UnlistenFn | undefined;
    let unlistenErr: UnlistenFn | undefined;

    listen<AiMessageEvent>("ai_message", (e) => {
      const p = e.payload.payload;
      setMessages((m) => {
        // 移除最后一个 thinking 占位
        const withoutThinking = m.filter((_, idx) =>
          !(m[idx].thinking && idx === m.length - 1)
        );
        return [
          ...withoutThinking,
          {
            role: p.status === "error" ? "system" : "ai",
            text: p.content ?? "",
            summary: p.summary,
            error: p.status === "error",
          },
        ];
      });
    }).then((f) => (unlistenMsg = f));

    listen<AiStatusEvent>("ai_status", (e) => {
      if (e.payload.status === "thinking") {
        setMessages((m) => {
          // 已有 thinking 占位就不重复加
          if (m.length > 0 && m[m.length - 1].thinking) return m;
          return [...m, { role: "ai", text: "", thinking: true }];
        });
      }
    }).then((f) => (unlistenStatus = f));

    listen<AiErrorEvent>("ai_error", (e) => {
      setMessages((m) => [
        ...m,
        { role: "system", text: `AI 错误：${e.payload.message}`, error: true },
      ]);
    }).then((f) => (unlistenErr = f));

    return () => {
      unlistenMsg?.();
      unlistenStatus?.();
      unlistenErr?.();
    };
  }, []);

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
      await invoke("send_message", { text, sessionId: null });
      // 响应通过 ai_message / ai_status / ai_error 事件回传
    } catch (e) {
      setMessages((m) => [
        ...m,
        { role: "system", text: `发送失败：${e}`, error: true },
      ]);
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

          <h2 className="section-title" style={{ marginTop: 20 }}>AI 服务配置</h2>
          <p style={{ fontSize: 11, color: "#8e8e93", marginTop: 0 }}>
            兼容 OpenAI 协议的任意服务 — 下方有常用 provider 快捷填入
          </p>

          <div className="field">
            <label>Base URL</label>
            <input
              type="text"
              placeholder="https://api.deepseek.com/v1"
              value={config.ai_base_url}
              onChange={(e) =>
                setConfig({ ...config, ai_base_url: e.target.value })
              }
            />
          </div>

          <div className="field">
            <label>API Key</label>
            <input
              type="password"
              placeholder="sk-xxx"
              value={config.ai_api_key}
              onChange={(e) =>
                setConfig({ ...config, ai_api_key: e.target.value })
              }
            />
          </div>

          <div className="field">
            <label>Model</label>
            <input
              type="text"
              placeholder="deepseek-chat / gpt-4o / glm-4.6 / ..."
              value={config.ai_model}
              onChange={(e) =>
                setConfig({ ...config, ai_model: e.target.value })
              }
            />
          </div>

          <details style={{ marginBottom: 14 }}>
            <summary style={{ fontSize: 11, color: "#8e8e93", cursor: "pointer" }}>
              快捷填入常用 Provider
            </summary>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 6, marginTop: 8 }}>
              {PROVIDER_PRESETS.map((p) => (
                <button
                  key={p.label}
                  className="btn secondary"
                  style={{ fontSize: 11, padding: "6px 8px" }}
                  onClick={() =>
                    setConfig({
                      ...config,
                      ai_base_url: p.base_url,
                      ai_model: p.model,
                    })
                  }
                >
                  {p.label}
                </button>
              ))}
            </div>
          </details>

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
              启动服务后，在下方输入消息，AI 回复会实时显示在这里，同时广播给已连接的设备。
            </div>
          )}
          {messages.map((m, i) => (
            <div
              key={i}
              className={`msg ${m.role === "user" ? "user" : m.role === "system" ? "system" : "ai"}${m.error ? " error" : ""}`}
            >
              {m.thinking ? (
                <em style={{ opacity: 0.6 }}>思考中...</em>
              ) : (
                <>
                  {m.summary && (
                    <div style={{ fontSize: 12, opacity: 0.7, marginBottom: 4 }}>
                      {m.summary}
                    </div>
                  )}
                  {m.text}
                </>
              )}
            </div>
          ))}
        </main>
      </div>

      <div className="composer">
        <input
          type="text"
          placeholder="输入消息，回车发送给 AI"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleSend()}
        />
        <button className="btn" onClick={handleSend}>发送</button>
      </div>
    </div>
  );
}
