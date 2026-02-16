// 系统托盘模块

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

/// 设置系统托盘
pub fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // 创建托盘菜单项
    let show_item = MenuItem::with_id(app, "show", "显示主界面", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "完全退出", true, None::<&str>)?;

    // 创建托盘菜单
    let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

    // 获取应用图标
    let icon = app.default_window_icon()
        .cloned()
        .expect("Failed to get default window icon");

    // 创建托盘图标
    let _tray = TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show" => {
                    // 显示主窗口
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    // 完全退出程序
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event {
                // 左键点击显示窗口
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
