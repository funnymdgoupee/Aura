//! 菜单栏常驻 — Tauri 2.x TrayIcon API
//!
//! 关闭主窗口后服务器继续后台运行，菜单栏显示状态
//! 点击左键切换窗口显隐

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, WindowEvent,
};

pub fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "隐藏到菜单栏", true, None::<&str>)?;
    let status = MenuItem::with_id(app, "status", "服务未启动", false, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出 Aura", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&status, &show, &hide, &quit])?;

    let _tray = TrayIconBuilder::with_id("aura-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

/// Mac 行为：点红圆是隐藏而不是退出；其他平台正常退出
pub fn on_window_event(window: &tauri::Window, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        #[cfg(target_os = "macos")]
        {
            let _ = window.hide();
            api.prevent_close();
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = window;
        }
    }
}
