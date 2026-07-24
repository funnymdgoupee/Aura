//! 桌面端 ↔ 手机端 WebSocket 通信协议
//!
//! 客户端 → 桌面端：ClientToServer
//! 桌面端 → 客户端：ServerToClient
//!
//! 协议与连接模式无关：局域网和中继模式下消息体完全一致，
//! 中继服务器只做盲转发，不解析也不修改消息内容。

use serde::{Deserialize, Serialize};

/// 客户端发给桌面端
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientToServer {
    /// 用户消息 / 快捷指令 / 文件 / 心跳
    Message {
        session_id: String,
        device_id: String,
        device_type: DeviceType,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        seq: Option<u64>,
        payload: ClientPayload,
        timestamp: i64,
    },
    /// 加入会话（首次连接时）
    Join {
        session_id: String,
        device_id: String,
        device_type: DeviceType,
        timestamp: i64,
    },
    /// 心跳保活
    Heartbeat {
        device_id: String,
        timestamp: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Iphone,
    Watch,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ClientPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<FileAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    pub name: String,
    /// base64-encoded
    pub data: String,
    pub mime_type: String,
}

/// 桌面端发给客户端
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerToClient {
    /// AI 响应消息（流式片段或最终消息）
    Message {
        session_id: String,
        seq: u64,
        from: MessageFrom,
        payload: ServerPayload,
        timestamp: i64,
    },
    /// 状态变化（思考中/执行中/完成/错误）
    Status {
        session_id: String,
        status: AiStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        timestamp: i64,
    },
    /// 全量历史同步（新设备加入时）
    Sync {
        session_id: String,
        messages: Vec<SyncMessage>,
        timestamp: i64,
    },
    /// 错误响应
    Error {
        session_id: String,
        code: String,
        message: String,
        timestamp: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageFrom {
    Ai,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ServerPayload {
    /// AI 响应内容（可能是流式片段）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// AI 主动生成的摘要（手表端优先显示）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<AiStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AiStatus {
    Thinking,
    Executing,
    Done,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub seq: u64,
    pub created_at: i64,
}

pub fn timestamp_now() -> i64 {
    chrono::Utc::now().timestamp()
}
