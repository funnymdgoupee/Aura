pub mod ai;
pub mod commands;
pub mod config;
pub mod db;
pub mod network;
pub mod pairing;
pub mod protocol;
pub mod server;
pub mod session;
pub mod tray;

use std::sync::Arc;

use tauri::Manager;
use tokio::sync::Mutex;

use crate::config::AppConfig;
use crate::db::Db;
use crate::protocol::ClientToServer;
use crate::session::SessionStore;

/// 全局应用状态，由 Tauri 管理
pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub transport: Arc<Mutex<Option<crate::network::TransportHandle>>>,
    /// Mac 前端 → router 的消息通道；router 启动时填入，停止时清空
    pub local_tx: Arc<Mutex<Option<tokio::sync::mpsc::UnboundedSender<ClientToServer>>>>,
    pub sessions: SessionStore,
}

#[cfg_attr(mobile, tauri::mobile_entry_point))]
pub fn run() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let handle = app.handle().clone();

            tauri::async_runtime::block_on(async move {
                let db_dir = handle
                    .path()
                    .app_data_dir()
                    .unwrap_or_else(|_| std::path::PathBuf::from("."));
                let db_path = db_dir.join("aura.db");
                let db = match Db::open(db_path).await {
                    Ok(db) => db,
                    Err(e) => {
                        log::error!("数据库初始化失败: {}", e);
                        return;
                    }
                };

                let config = AppConfig::load_or_default(&handle)
                    .await
                    .unwrap_or_default();

                let state = AppState {
                    config: Arc::new(Mutex::new(config)),
                    transport: Arc::new(Mutex::new(None)),
                    local_tx: Arc::new(Mutex::new(None)),
                    sessions: SessionStore::new(),
                };
                let _ = handle.manage(state);
                let _ = handle.manage(db);
            });

            let _ = tray::build_tray(app.handle());
            Ok(())
        })
        .on_window_event(tray::on_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::start_with_mode,
            commands::stop_transport,
            commands::get_transport_status,
            commands::generate_pairing_qr,
            commands::switch_connection_mode,
            commands::get_config,
            commands::save_config,
            commands::send_test_message,
            commands::send_message,
            commands::list_sessions,
            commands::clear_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
