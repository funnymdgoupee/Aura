//! 会话存储 — Markdown 文件持久化
//!
//! 每个会话一个 .md 文件，存放于 `base_dir/<session_id>.md`
//! 格式：YAML frontmatter（手解析）+ Markdown body（按 `## timestamp — role` 分段）
//!
//! 内存层：append 即写盘，list 扫目录；get_or_create 读盘缓存到 HashMap
//! 这样兼顾"用户可读文件"+"App 内查询快"

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::fs;
use tokio::sync::{Mutex, RwLock};

use crate::ai::ChatMessage;

/// 会话元数据 — 列表展示用
#[derive(Debug, Clone, Serialize)]
pub struct SessionMeta {
    pub id: String,
    pub title: String,
    pub preview: String,
    pub updated_at: String,
    pub message_count: usize,
}

#[derive(Clone)]
pub struct SessionStore {
    base_dir: Arc<RwLock<PathBuf>>,
    /// 内存缓存 — 启动后已读过的会话在此，避免重复 IO
    cache: Arc<Mutex<HashMap<String, Vec<ChatMessage>>>>,
}

impl SessionStore {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir: Arc::new(RwLock::new(base_dir)),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 用户在设置里改了 storage_dir 时调用
    pub async fn set_base_dir(&self, new_dir: PathBuf) {
        let mut guard = self.base_dir.write().await;
        *guard = new_dir;
        self.cache.lock().await.clear();
    }

    async fn file_path(&self, session_id: &str) -> PathBuf {
        let guard = self.base_dir.read().await;
        guard.join(format!("{}.md", sanitize_filename(session_id)))
    }

    /// 读取或创建会话 — 若文件存在则解析，否则返回空 Vec
    pub async fn get_or_create(&self, session_id: &str) -> Vec<ChatMessage> {
        {
            let cache = self.cache.lock().await;
            if let Some(msgs) = cache.get(session_id) {
                return msgs.clone();
            }
        }

        let path = self.file_path(session_id).await;
        if !path.exists() {
            return Vec::new();
        }
        let Ok(text) = fs::read_to_string(&path).await else {
            return Vec::new();
        };
        let msgs = parse_markdown(&text);
        self.cache
            .lock()
            .await
            .insert(session_id.to_string(), msgs.clone());
        msgs
    }

    /// 追加一条消息到会话 — 同时写文件 + 更新缓存
    pub async fn append(&self, session_id: &str, message: ChatMessage) {
        // 先写盘（失败也继续，缓存还是要更新）
        let _ = self.append_to_file(session_id, &message).await;

        // 更新缓存
        let mut cache = self.cache.lock().await;
        cache
            .entry(session_id.to_string())
            .or_insert_with(Vec::new)
            .push(message);
    }

    async fn append_to_file(&self, session_id: &str, message: &ChatMessage) -> anyhow::Result<()> {
        let path = self.file_path(session_id).await;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.ok();
        }

        let now = Utc::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let body_section = format_message_section(&timestamp, message);

        if !path.exists() {
            // 新文件 — 写 frontmatter + 第一段
            let title = derive_title(message);
            let frontmatter = format_frontmatter(session_id, &title, &now);
            let content = format!("{}\n\n{}", frontmatter, body_section);
            fs::write(&path, content).await?;
        } else {
            // 已存在 — 追加段落，并更新 updated_at frontmatter
            let existing = fs::read_to_string(&path).await?;
            let updated = update_frontmatter_updated_at(&existing, &now);
            let new_content = format!("{}\n\n{}", updated.trim_end(), body_section);
            fs::write(&path, new_content).await?;
        }
        Ok(())
    }

    /// 列出所有会话 — 扫目录，解析 frontmatter
    pub async fn list_sessions(&self) -> Vec<SessionMeta> {
        let dir = self.base_dir.read().await.clone();
        if !dir.exists() {
            return Vec::new();
        }
        let mut entries = match fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };
        let mut metas = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            if let Some(meta) = parse_session_meta(&path).await {
                metas.push(meta);
            }
        }
        // 按 updated_at 倒序
        metas.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        metas
    }

    /// 清空会话 — 删除文件 + 缓存
    pub async fn clear(&self, session_id: &str) {
        let path = self.file_path(session_id).await;
        fs::remove_file(&path).await.ok();
        self.cache.lock().await.remove(session_id);
    }
}

// ============================================
// Markdown 解析 / 生成
// ============================================

/// 文件名安全化 — 非 [a-zA-Z0-9_-] 替换为 _
fn sanitize_filename(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn format_frontmatter(session_id: &str, title: &str, created_at: &DateTime<Utc>) -> String {
    format!(
        "---\nsession_id: {}\ntitle: {}\ncreated_at: {}\nupdated_at: {}\nmessage_count: 0\n---",
        session_id,
        escape_yaml(title),
        created_at.to_rfc3339(),
        created_at.to_rfc3339(),
    )
}

fn update_frontmatter_updated_at(content: &str, now: &DateTime<Utc>) -> String {
    // 简单替换：找 updated_at: ... 行
    let new_line = format!("updated_at: {}", now.to_rfc3339());
    let mut lines = content.lines().collect::<Vec<_>>();
    let mut in_frontmatter = false;
    let mut count = 0usize;
    for line in &mut lines {
        if *line == "---" {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if in_frontmatter {
            if line.starts_with("updated_at:") {
                *line = &new_line;
            } else if line.starts_with("message_count:") {
                // 留待稍后重写
            }
        }
        if !in_frontmatter && line.starts_with("## ") {
            count += 1;
        }
    }
    // 重写 message_count
    let mut in_fm = false;
    let count_str = format!("message_count: {}", count);
    for line in &mut lines {
        if *line == "---" {
            in_fm = !in_fm;
            continue;
        }
        if in_fm && line.starts_with("message_count:") {
            *line = &count_str;
        }
    }
    lines.join("\n")
}

fn format_message_section(timestamp: &str, message: &ChatMessage) -> String {
    let role = match message.role.as_str() {
        "assistant" => "ai".to_string(),
        other => other.to_string(),
    };
    let mut out = format!("## {} — {}\n{}", timestamp, role, message.content);
    // 注意：summary 概念在 chat 历史里不持久化（thinking 状态是临时的，不写盘）
    out.push_str("\n");
    out
}

fn derive_title(message: &ChatMessage) -> String {
    let text = message.content.trim();
    if text.is_empty() {
        return "新会话".to_string();
    }
    let limit = 30;
    let title: String = text.chars().take(limit).collect();
    if text.chars().count() > limit {
        format!("{}…", title)
    } else {
        title
    }
}

fn escape_yaml(s: &str) -> String {
    // 简化：只要不包含特殊字符就直接返回；含 : 或 # 就加引号
    if s.contains(':') || s.contains('#') || s.contains('"') {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s.to_string()
    }
}

/// 解析整个 Markdown 文件，返回消息列表（不含 frontmatter）
fn parse_markdown(text: &str) -> Vec<ChatMessage> {
    let body = strip_frontmatter(text);
    let mut msgs = Vec::new();
    let mut current_role: Option<String> = None;
    let mut current_content = String::new();

    for line in body.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            // 命中段落头 `## timestamp — role`
            if let Some(role) = current_role.take() {
                if !current_content.is_empty() {
                    msgs.push(ChatMessage {
                        role: normalize_role(&role),
                        content: current_content.trim().to_string(),
                    });
                }
            }
            // 提取 role：在最后一个 — 后
            let role = rest.split("—").last().map(|s| s.trim().to_string()).unwrap_or_default();
            current_role = Some(role);
            current_content.clear();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }
    if let Some(role) = current_role.take() {
        if !current_content.is_empty() {
            msgs.push(ChatMessage {
                role: normalize_role(&role),
                content: current_content.trim().to_string(),
            });
        }
    }
    msgs
}

fn strip_frontmatter(text: &str) -> String {
    let trimmed = text.trim_start();
    if !trimmed.starts_with("---") {
        return text.to_string();
    }
    let after_first = &trimmed[3..];
    if let Some(end) = after_first.find("\n---") {
        let body_start = 3 + end + 4; // 跳过开头的 --- 和结尾的 ---
        let body = &trimmed[body_start..];
        return body.trim_start_matches('\n').to_string();
    }
    text.to_string()
}

fn normalize_role(r: &str) -> String {
    match r {
        "ai" | "assistant" => "assistant".to_string(),
        "user" => "user".to_string(),
        "system" => "system".to_string(),
        other => other.to_string(),
    }
}

async fn parse_session_meta(path: &PathBuf) -> Option<SessionMeta> {
    let text = fs::read_to_string(path).await.ok()?;
    let (id, title, updated_at) = parse_frontmatter(&text)?;
    let body = strip_frontmatter(&text);
    let messages = parse_markdown(&text);
    let preview = messages
        .last()
        .map(|m| {
            let content = m.content.trim();
            let limit = 40;
            let s: String = content.chars().take(limit).collect();
            if content.chars().count() > limit {
                format!("{}…", s)
            } else {
                s
            }
        })
        .unwrap_or_default();
    Some(SessionMeta {
        id,
        title: title.unwrap_or_else(|| "新会话".to_string()),
        preview,
        updated_at: updated_at.unwrap_or_default(),
        message_count: messages.len(),
    })
}

/// 从 frontmatter 提取 session_id / title / updated_at
fn parse_frontmatter(text: &str) -> Option<(String, Option<String>, Option<String>)> {
    let trimmed = text.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let after_first = &trimmed[3..];
    let end = after_first.find("\n---")?;
    let frontmatter = &after_first[..end];
    let mut id = None;
    let mut title = None;
    let mut updated = None;
    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("session_id:") {
            id = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("title:") {
            title = Some(v.trim().trim_matches('"').to_string());
        } else if let Some(v) = line.strip_prefix("updated_at:") {
            updated = Some(v.trim().to_string());
        }
    }
    Some((id?, title, updated))
}
