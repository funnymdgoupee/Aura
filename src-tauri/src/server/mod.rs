//! WebSocket 服务器实现（局域网模式专用）
//!
//! 由 LanTransport 调用，不直接对业务层暴露

pub mod connections;
pub mod router;

pub use connections::ConnectionManager;
