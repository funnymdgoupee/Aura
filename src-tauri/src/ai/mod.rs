//! 通用 OpenAI 兼容 Chat Completions 客户端
//!
//! 用户可在前端配置任意 OpenAI 协议服务：
//!   - DeepSeek：https://api.deepseek.com/v1
//!   - OpenAI：https://api.openai.com/v1
//!   - Anthropic（OpenAI 兼容代理）：https://api.anthropic.com/v1/openai
//!   - Gemini：https://generativelanguage.googleapis.com/v1beta/openai
//!   - GLM：https://open.bigmodel.cn/api/paas/v4
//!   - Qwen：https://dashscope.aliyuncs.com/compatible-mode/v1
//!   - Kimi：https://api.moonshot.cn/v1
//!   - Doubao：https://ark.cn-beijing.volces.com/api/v3
//!   - Ollama：http://localhost:11434/v1
//!   - 自定义：用户自填 base_url + model
//!
//! 手表模式：通过 system prompt 要求模型同时输出 {"summary": "...", "content": "..."}，
//! 我们解析后分别传给手表端。

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::AppConfig;

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub session_id: String,
    pub messages: Vec<ChatMessage>,
    pub model: String,
    /// 是否目标是手表端 — 是的话要求输出 summary + content 的 JSON
    pub watch_target: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub summary: Option<String>,
}

pub struct AiClient {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl AiClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            http: reqwest::Client::builder()
                .build()
                .expect("reqwest client"),
            base_url,
            api_key,
        }
    }

    pub fn from_config(config: &AppConfig) -> Self {
        Self::new(config.ai_base_url.clone(), config.ai_api_key.clone())
    }

    /// 非流式调用 — Phase 1 简化版，后续可改为流式
    ///
    /// 1. POST {base_url}/chat/completions
    /// 2. body: {"model": ..., "messages": [...], "stream": false}
    /// 3. watch_target=true 时 system prompt 要求 JSON 输出
    /// 4. 解析响应：普通模式取 choices[0].message.content
    ///    手表模式解析 JSON 取 summary + content
    pub async fn chat(&self, mut req: ChatRequest) -> anyhow::Result<ChatResponse> {
        let url = format!(
            "{}/chat/completions",
            self.base_url.trim_end_matches('/')
        );

        if req.watch_target {
            req.messages.insert(
                0,
                ChatMessage {
                    role: "system".into(),
                    content: WATCH_SYSTEM_PROMPT.into(),
                },
            );
        }

        let body = serde_json::json!({
            "model": req.model,
            "messages": req.messages,
            "stream": false,
        });

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "AI API 返回 {} 状态码: {}",
                status,
                text.chars().take(500).collect::<String>()
            ));
        }

        let v: Value = serde_json::from_str(&text)?;
        let content = v["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("AI 响应缺少 content: {}", text.chars().take(200).collect::<String>()))?
            .to_string();

        let summary = if req.watch_target {
            parse_watch_summary(&content)
        } else {
            None
        };

        // 手表模式：如果成功解析出 summary，content 字段需要从外层 JSON 中提取（避免外层 JSON 被当成内容显示）
        let final_content = if req.watch_target && summary.is_some() {
            parse_watch_content(&content).unwrap_or(content)
        } else {
            content
        };

        Ok(ChatResponse {
            content: final_content,
            summary,
        })
    }

    /// 流式调用 — 异步迭代器风格
    /// 每收到一个 chunk 调用一次 on_chunk(content_delta)
    /// 完成后返回完整的 content + summary
    pub async fn chat_stream<F>(
        &self,
        mut req: ChatRequest,
        mut on_chunk: F,
    ) -> anyhow::Result<ChatResponse>
    where
        F: FnMut(&str),
    {
        let url = format!(
            "{}/chat/completions",
            self.base_url.trim_end_matches('/')
        );

        if req.watch_target {
            req.messages.insert(
                0,
                ChatMessage {
                    role: "system".into(),
                    content: WATCH_SYSTEM_PROMPT.into(),
                },
            );
        }

        let body = serde_json::json!({
            "model": req.model,
            "messages": req.messages,
            "stream": true,
        });

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await?;
            return Err(anyhow::anyhow!(
                "AI API 返回 {} 状态码: {}",
                status,
                text.chars().take(500).collect::<String>()
            ));
        }

        use futures_util::StreamExt;
        let mut stream = resp.bytes_stream();
        let mut full_content = String::new();
        let mut buf = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            buf.push_str(&String::from_utf8_lossy(&chunk));

            // SSE 帧以 \n\n 分隔，逐行处理
            while let Some(pos) = buf.find('\n') {
                let line: String = buf.drain(..=pos).collect();
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let Some(json_str) = line.strip_prefix("data: ") else {
                    continue;
                };
                if json_str.trim() == "[DONE]" {
                    continue;
                }
                let v: Value = match serde_json::from_str(json_str) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if let Some(delta) = v["choices"][0]["delta"]["content"].as_str() {
                    if !delta.is_empty() {
                        full_content.push_str(delta);
                        on_chunk(delta);
                    }
                }
            }
        }

        let summary = if req.watch_target {
            parse_watch_summary(&full_content)
        } else {
            None
        };
        let final_content = if req.watch_target && summary.is_some() {
            parse_watch_content(&full_content).unwrap_or(full_content)
        } else {
            full_content
        };

        Ok(ChatResponse {
            content: final_content,
            summary,
        })
    }
}

/// 手表模式 system prompt — 要求模型输出 {"summary": "...", "content": "..."} JSON
pub const WATCH_SYSTEM_PROMPT: &str = r#"
你是用户的 AI 助手。当用户通过 Apple Watch 与你交互时：
1. 先给出一句 30 字以内的核心结论作为 summary
2. 然后给出详细回复作为 content
3. 格式要求：必须输出合法 JSON：{"summary": "核心结论", "content": "详细回复"}
4. 不要在 JSON 外添加任何额外文字、代码块标记或解释
5. 如果回复包含代码、路径、关键数据，确保 summary 中包含最关键的信息
"#;

/// 尝试从模型输出中解析出 summary 字段
/// 模型可能直接输出 JSON，也可能包在 ```json ... ``` 中
fn parse_watch_summary(raw: &str) -> Option<String> {
    parse_watch_json(raw).map(|v| v.summary)
}

/// 尝试从模型输出中解析出 content 字段
fn parse_watch_content(raw: &str) -> Option<String> {
    parse_watch_json(raw).map(|v| v.content)
}

#[derive(Deserialize)]
struct WatchReply {
    summary: String,
    content: String,
}

fn parse_watch_json(raw: &str) -> Option<WatchReply> {
    let trimmed = raw.trim();
    // 去掉可能的 ```json ... ``` 包裹
    let candidate = if let Some(stripped) = trimmed
        .strip_prefix("```json")
        .and_then(|s| s.strip_suffix("```"))
    {
        stripped.trim()
    } else if let Some(stripped) = trimmed
        .strip_prefix("```")
        .and_then(|s| s.strip_suffix("```"))
    {
        stripped.trim()
    } else {
        trimmed
    };
    serde_json::from_str(candidate).ok()
}
