import { useEffect, useState, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

type ConnectionMode = "lan" | "relay";
type View = "welcome" | "chat" | "settings";

interface AppConfig {
  ai_base_url: string;
  ai_api_key: string;
  ai_model: string;
  server_port: number;
  connection_mode: ConnectionMode;
  relay_server_url: string;
  relay_room_id: string;
  relay_secret_key: string;
  storage_dir: string | null;
}

interface ProviderPreset {
  label: string;
  base_url: string;
  model: string;
}

const PROVIDER_PRESETS: ProviderPreset[] = [
  { label: "DeepSeek", base_url: "https://api.deepseek.com/v1", model: "deepseek-chat" },
  { label: "OpenAI", base_url: "https://api.openai.com/v1", model: "gpt-4o" },
  { label: "Claude", base_url: "https://api.anthropic.com/v1/openai", model: "claude-sonnet-4-6" },
  { label: "Gemini", base_url: "https://generativelanguage.googleapis.com/v1beta/openai", model: "gemini-2.5-flash" },
  { label: "智谱 GLM", base_url: "https://open.bigmodel.cn/api/paas/v4", model: "glm-4.6" },
  { label: "通义 Qwen", base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1", model: "qwen3-coder-plus" },
  { label: "Kimi", base_url: "https://api.moonshot.cn/v1", model: "kimi-k2" },
  { label: "豆包", base_url: "https://ark.cn-beijing.volces.com/api/v3", model: "doubao-1-5-pro" },
  { label: "Ollama", base_url: "http://localhost:11434/v1", model: "qwen2.5" },
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

interface SessionInfo {
  id: string;
  title: string;
  preview: string;
  updated_at: string;
  message_count: number;
}

const QUICK_CARDS = [
  { icon: "✦", title: "写作助手", desc: "邮件、文档、文案润色", prompt: "帮我写一封简洁的商务邮件，主题是关于项目进度同步" },
  { icon: "◈", title: "代码助手", desc: "调试、重构、解释代码", prompt: "解释一下 Rust 的所有权系统" },
  { icon: "◉", title: "知识问答", desc: "概念解释、对比分析", prompt: "对比一下局域网和中继服务器的优缺点" },
];

function formatUpdated(rfc3339: string): string {
  if (!rfc3339) return "—";
  const d = new Date(rfc3339);
  if (isNaN(d.getTime())) return rfc3339;
  const now = new Date();
  const sameDay = d.toDateString() === now.toDateString();
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  if (sameDay) return `${hh}:${mm}`;
  const mo = String(d.getMonth() + 1).padStart(2, "0");
  const da = String(d.getDate()).padStart(2, "0");
  return `${mo}-${da}`;
}

export default function App() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [status, setStatus] = useState<StatusResponse | null>(null);
  const [qr, setQr] = useState<PairingResult | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState("");
  const [view, setView] = useState<View>("welcome");
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [activeSession, setActiveSession] = useState<string>("mac-local");
  const [sessionSearch, setSessionSearch] = useState("");
  const messagesEndRef = useRef<HTMLDivElement | null>(null);

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

  const refreshSessions = useCallback(async () => {
    try {
      const metas = await invoke<SessionInfo[]>("list_sessions");
      setSessions(metas);
    } catch {
      setSessions([]);
    }
  }, []);

  useEffect(() => {
    refreshConfig();
    refreshStatus();
    refreshSessions();
    const timer = setInterval(refreshStatus, 3000);
    return () => clearInterval(timer);
  }, [refreshConfig, refreshStatus, refreshSessions]);

  useEffect(() => {
    let unlistenMsg: UnlistenFn | undefined;
    let unlistenStatus: UnlistenFn | undefined;
    let unlistenErr: UnlistenFn | undefined;

    listen<AiMessageEvent>("ai_message", (e) => {
      const p = e.payload.payload;
      setMessages((m) => {
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

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

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

  const handleSend = async (text?: string) => {
    const content = (text ?? input).trim();
    if (!content) return;
    setMessages((m) => [...m, { role: "user", text: content }]);
    setInput("");
    setView("chat");
    try {
      await invoke("send_message", { text: content, sessionId: activeSession });
    } catch (e) {
      setMessages((m) => [
        ...m,
        { role: "system", text: `发送失败：${e}`, error: true },
      ]);
    }
  };

  const handleNewSession = () => {
    const id = `s-${Date.now()}`;
    setActiveSession(id);
    setMessages([]);
    setView("chat");
    refreshSessions();
  };

  const handlePickStorageDir = async () => {
    try {
      const picked = await invoke<string | null>("pick_storage_folder");
      if (!picked) return;
      if (!config) return;
      const updated = { ...config, storage_dir: picked };
      setConfig(updated);
      await invoke("save_config", { config: updated });
      await invoke("set_storage_dir", { dir: picked });
      refreshSessions();
    } catch (e) {
      alert("选择失败: " + e);
    }
  };

  const handleResetStorageDir = async () => {
    if (!config) return;
    const updated = { ...config, storage_dir: null };
    setConfig(updated);
    await invoke("save_config", { config: updated });
    await invoke("set_storage_dir", { dir: null });
    refreshSessions();
  };

  const handleClearSession = async (id: string) => {
    try {
      await invoke("clear_session", { sessionId: id });
      if (id === activeSession) setMessages([]);
      refreshSessions();
    } catch (e) {
      alert("清空失败: " + e);
    }
  };

  if (!config) {
    return (
      <>
        <div className="ambient-glow">
          <div className="glow-orb glow-orb-1" />
          <div className="glow-orb glow-orb-2" />
          <div className="glow-orb glow-orb-3" />
        </div>
        <div className="grid-texture" />
        <div style={{ display: "flex", height: "100vh", alignItems: "center", justifyContent: "center", color: "var(--text-muted)" }}>
          加载中...
        </div>
      </>
    );
  }

  const running = status?.status === "listening" || status?.status === "connected";
  const statusClass = running ? "online" : status?.status === "error" ? "offline" : "offline";
  const statusText = status ? `${status.mode} · ${status.status}` : "未启动";

  const filteredSessions = sessions.filter((s) =>
    s.title.toLowerCase().includes(sessionSearch.toLowerCase())
  );

  return (
    <>
      <div className="ambient-glow">
        <div className="glow-orb glow-orb-1" />
        <div className="glow-orb glow-orb-2" />
        <div className="glow-orb glow-orb-3" />
      </div>
      <div className="grid-texture" />

      <div className={`app-shell${view === "welcome" ? " no-panel" : ""}`}>
        {/* ============ Sidebar ============ */}
        <aside className="sidebar">
          <div className="brand">
            <div className="brand-logo">A</div>
            <div className="brand-name">Aura</div>
          </div>

          <div className="nav-section-title">主功能</div>
          <button
            className={`nav-item${view === "welcome" ? " active" : ""}`}
            onClick={() => setView("welcome")}
          >
            <span className="nav-icon">⌂</span>
            首页
          </button>
          <button
            className={`nav-item${view === "chat" ? " active" : ""}`}
            onClick={() => setView("chat")}
          >
            <span className="nav-icon">💬</span>
            聊天
          </button>
          <button
            className={`nav-item${view === "settings" ? " active" : ""}`}
            onClick={() => setView("settings")}
          >
            <span className="nav-icon">⚙</span>
            设置
          </button>

          <div className="nav-section-title">连接</div>
          <button className="nav-item" onClick={handleGenerateQr}>
            <span className="nav-icon">▦</span>
            配对二维码
          </button>
          <button className="nav-item" onClick={running ? handleStop : handleStart}>
            <span className="nav-icon">⏻</span>
            {running ? "停止服务" : "启动服务"}
          </button>

          <div className="sidebar-spacer" />

          <div className="user-card">
            <div className="user-avatar">U</div>
            <div className="user-info">
              <div className="user-name">本地用户</div>
              <div className="user-meta">{running ? "在线" : "离线"}</div>
            </div>
          </div>
        </aside>

        {/* ============ Main ============ */}
        <main className="main">
          <div className="topbar">
            <div className="topbar-left">
              <span className={`status-dot ${statusClass}`} />
              <span>{statusText}</span>
            </div>
            <div className="topbar-right">
              {running ? (
                <button className="btn btn-secondary" onClick={handleStop}>停止</button>
              ) : (
                <button className="btn btn-primary" onClick={handleStart}>启动服务</button>
              )}
            </div>
          </div>

          <div className="content">
            {view === "welcome" && (
              <div className="welcome">
                <div className="welcome-icon">✦</div>
                <h1 className="welcome-title">你好，我是 Aura</h1>
                <p className="welcome-subtitle">
                  你的私人 AI 助手。启动服务后即可在 Mac、iPhone、Apple Watch 之间同步对话。
                </p>
                <div className="quick-cards">
                  {QUICK_CARDS.map((c) => (
                    <button
                      key={c.title}
                      className="quick-card"
                      onClick={() => handleSend(c.prompt)}
                    >
                      <div className="quick-card-icon">{c.icon}</div>
                      <div className="quick-card-title">{c.title}</div>
                      <div className="quick-card-desc">{c.desc}</div>
                    </button>
                  ))}
                </div>
              </div>
            )}

            {view === "chat" && (
              <div className="chat">
                <div className="chat-messages">
                  {messages.length === 0 && (
                    <div className="chat-empty">
                      输入消息开始对话，AI 回复会同步到所有已配对设备。
                    </div>
                  )}
                  {messages.map((m, i) => {
                    if (m.thinking) {
                      return <div key={i} className="msg thinking" />;
                    }
                    const cls = `msg ${m.role === "user" ? "user" : m.role === "system" ? "system" : "ai"}${m.error ? " error" : ""}`;
                    return (
                      <div key={i} className={cls}>
                        {m.role === "ai" && m.summary && (
                          <div className="msg-summary">{m.summary}</div>
                        )}
                        <div className="msg-content">
                          {m.role === "ai" ? (
                            <ReactMarkdown remarkPlugins={[remarkGfm]}>
                              {m.text}
                            </ReactMarkdown>
                          ) : (
                            m.text
                          )}
                        </div>
                      </div>
                    );
                  })}
                  <div ref={messagesEndRef} />
                </div>

                <div className="composer">
                  <div className="composer-input-wrap">
                    <textarea
                      className="composer-input"
                      placeholder="输入消息，回车发送，Shift+Enter 换行"
                      value={input}
                      rows={1}
                      onChange={(e) => setInput(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter" && !e.shiftKey) {
                          e.preventDefault();
                          handleSend();
                        }
                      }}
                    />
                    <div className="composer-actions">
                      <button className="icon-btn" title="附件（Phase 2）">📎</button>
                      <button
                        className="send-btn"
                        onClick={() => handleSend()}
                        disabled={!input.trim()}
                        title="发送"
                      >↑</button>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {view === "settings" && (
              <div className="settings">
                <h1 className="settings-title">设置</h1>
                <p className="settings-subtitle">
                  配置 AI 服务、连接模式与设备配对。
                </p>

                <div className="settings-section">
                  <h3 className="settings-section-title">AI 服务</h3>
                  <div className="field">
                    <label className="field-label">Base URL</label>
                    <input
                      className="field-input"
                      placeholder="https://api.deepseek.com/v1"
                      value={config.ai_base_url}
                      onChange={(e) => setConfig({ ...config, ai_base_url: e.target.value })}
                    />
                    <span className="field-hint">兼容 OpenAI 协议的任意服务</span>
                  </div>
                  <div className="field">
                    <label className="field-label">API Key</label>
                    <input
                      className="field-input"
                      type="password"
                      placeholder="sk-xxx"
                      value={config.ai_api_key}
                      onChange={(e) => setConfig({ ...config, ai_api_key: e.target.value })}
                    />
                  </div>
                  <div className="field">
                    <label className="field-label">Model</label>
                    <input
                      className="field-input"
                      placeholder="deepseek-chat / gpt-4o / glm-4.6"
                      value={config.ai_model}
                      onChange={(e) => setConfig({ ...config, ai_model: e.target.value })}
                    />
                  </div>
                  <div className="field">
                    <label className="field-label">快捷 Provider</label>
                    <div className="preset-grid">
                      {PROVIDER_PRESETS.map((p) => (
                        <button
                          key={p.label}
                          className="preset-btn"
                          onClick={() => setConfig({
                            ...config,
                            ai_base_url: p.base_url,
                            ai_model: p.model,
                          })}
                        >
                          {p.label}
                        </button>
                      ))}
                    </div>
                  </div>
                </div>

                <div className="settings-section">
                  <h3 className="settings-section-title">连接模式</h3>
                  <div className="field">
                    <label className="field-label">模式</label>
                    <select
                      className="field-select"
                      value={config.connection_mode}
                      onChange={(e) => handleSwitchMode(e.target.value as ConnectionMode)}
                    >
                      <option value="lan">局域网（Mac 作为服务器）</option>
                      <option value="relay">自定义服务器（中继，Phase 4）</option>
                    </select>
                  </div>
                  {config.connection_mode === "lan" ? (
                    <div className="field">
                      <label className="field-label">监听端口</label>
                      <input
                        className="field-input"
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
                        <label className="field-label">中继服务器地址</label>
                        <input
                          className="field-input"
                          placeholder="wss://your-vps.com/relay"
                          value={config.relay_server_url}
                          onChange={(e) => setConfig({ ...config, relay_server_url: e.target.value })}
                        />
                      </div>
                      <div className="field">
                        <label className="field-label">Room ID</label>
                        <input
                          className="field-input"
                          value={config.relay_room_id}
                          onChange={(e) => setConfig({ ...config, relay_room_id: e.target.value })}
                        />
                      </div>
                      <div className="field">
                        <label className="field-label">Secret Key</label>
                        <input
                          className="field-input"
                          type="password"
                          value={config.relay_secret_key}
                          onChange={(e) => setConfig({ ...config, relay_secret_key: e.target.value })}
                        />
                        <span className="field-hint">中继模式 Phase 4 实现，当前仅作配置占位</span>
                      </div>
                    </>
                  )}
                </div>

                <div className="settings-section">
                  <h3 className="settings-section-title">配对二维码</h3>
                  <button className="btn btn-secondary" onClick={handleGenerateQr}>
                    生成二维码
                  </button>
                  {qr && (
                    <>
                      <img src={qr.qr_data_url} alt="pairing qr" className="qr-preview" />
                      <div className="qr-url">{qr.raw_url}</div>
                    </>
                  )}
                </div>

                <div className="settings-section">
                  <h3 className="settings-section-title">存储位置</h3>
                  <div className="field">
                    <label className="field-label">会话历史目录</label>
                    <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
                      <input
                        className="field-input"
                        style={{ flex: 1 }}
                        readOnly
                        placeholder="（默认：~/Library/Application Support/com.aura.app/sessions/）"
                        value={config.storage_dir ?? ""}
                      />
                      <button className="btn btn-secondary" onClick={handlePickStorageDir}>
                        选择文件夹
                      </button>
                      {config.storage_dir && (
                        <button className="btn btn-ghost" onClick={handleResetStorageDir}>
                          重置
                        </button>
                      )}
                    </div>
                    <span className="field-hint">
                      每个会话保存为一个 .md 文件，可直接用编辑器打开。可指向 iCloud Drive / Dropbox 做跨设备备份。
                    </span>
                  </div>
                </div>

                <div className="btn-row">
                  <button className="btn btn-primary" onClick={handleSaveConfig}>保存配置</button>
                </div>
              </div>
            )}
          </div>
        </main>

        {/* ============ Right Panel ============ */}
        {view !== "welcome" && (
          <aside className="panel">
            <div className="panel-header">
              <h3 className="panel-title">历史会话</h3>
              <button className="icon-btn" title="刷新" onClick={refreshSessions}>↻</button>
            </div>
            <input
              className="search-input"
              placeholder="搜索会话..."
              value={sessionSearch}
              onChange={(e) => setSessionSearch(e.target.value)}
            />
            <div className="session-list">
              {filteredSessions.length === 0 && (
                <div style={{ fontSize: 12, color: "var(--text-muted)", textAlign: "center", padding: "20px 0" }}>
                  暂无会话
                </div>
              )}
              {filteredSessions.map((s) => (
                <div
                  key={s.id}
                  className={`session-item${s.id === activeSession ? " active" : ""}`}
                  onClick={() => {
                    setActiveSession(s.id);
                    setMessages([]);
                    setView("chat");
                  }}
                >
                  <div className="session-item-title">{s.title}</div>
                  <div className="session-item-desc">{s.preview}</div>
                  <div className="session-item-time">
                    {formatUpdated(s.updated_at)} · {s.message_count} 条
                  </div>
                </div>
              ))}
            </div>
            <button className="new-session-btn" onClick={handleNewSession}>
              + 新建会话
            </button>
          </aside>
        )}
      </div>
    </>
  );
}
