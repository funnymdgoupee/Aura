//! DeepSeek API 调用封装（stub）
//!
//! Phase 1 实现：流式对话、多轮上下文、手表模式摘要生成
//! 当前为骨架，确保调用签名稳定

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub session_id: String,
    pub messages: Vec<ChatMessage>,
    pub model: String,
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
    api_key: String,
}

impl AiClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    /// Phase 1 实现：调用 DeepSeek API，流式返回
    /// 手表模式下通过 system prompt 要求模型同时输出 summary
    pub async fn chat(&self, _req: ChatRequest) -> anyhow::Result<ChatResponse> {
        // TODO Phase 1:
        // 1. 构造 system_prompt（如果是 watch_target，要求输出 {"summary":..., "content":...}）
        // 2. 调用 https://api.deepseek.com/chat/completions，stream=true
        // 3. 用 reqwest 流式读取 SSE
        // 4. 解析 JSON：{"summary": "...", "content": "..."}
        // 5. 返回 ChatResponse

        let _ = &self.api_key;
        Err(anyhow::anyhow!("AiClient 尚未实现（Phase 1）"))
    }
}

/// 手表模式下的 system prompt
pub const WATCH_SYSTEM_PROMPT: &str = r#"
你是用户的 AI 助手。当用户通过 Apple Watch 与你交互时：
1. 先给出一句 30 字以内的核心结论作为 summary
2. 然后给出详细回复作为 content
3. 格式要求：{"summary": "核心结论", "content": "详细回复"}
4. 如果回复包含代码、路径、关键数据，确保 summary 中包含最关键的信息
"#;
