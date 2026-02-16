// CUFE-COURSE 主入口模块
// 按功能域拆分为多个子模块

mod models;
mod crypto;
mod client;
mod storage;
mod parser;

// 命令模块
mod commands;
// 业务逻辑服务
mod services;
// 系统功能
mod system;

use client::EduSystemState;
use std::sync::Arc;
use tauri::WindowEvent;
use storage::StorageManager;

// 重新导出命令模块中的函数，以便在 invoke_handler 中使用
use commands::auth;
use commands::schedule;
use commands::import_export;
use commands::exam;
use commands::config;
use commands::date;
use commands::background;
use commands::update;

// ============== Main Entry Point ==============

pub fn run() {
    let base_url = "https://xuanke.cufe.edu.cn/jwglxt".to_string();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .manage(Arc::new(EduSystemState::new(base_url)))
        .setup(|app| {
            // 初始化系统托盘
            if let Err(e) = system::tray::setup_tray(app) {
                eprintln!("Failed to setup tray: {}", e);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // 读取配置获取关闭行为
                let storage = match StorageManager::new() {
                    Ok(s) => s,
                    Err(_) => {
                        // 配置加载失败，默认最小化到托盘
                        api.prevent_close();
                        let _ = window.hide();
                        return;
                    }
                };

                let config = storage.load_config().unwrap_or_default();

                if config.close_action_minimize_to_tray.unwrap_or(true) {
                    // 默认：最小化到托盘
                    api.prevent_close();
                    let _ = window.hide();
                }
                // 否则：允许关闭，程序退出
            }
        })
        .invoke_handler(tauri::generate_handler![
            // 认证命令
            auth::login_and_get_user_info,
            auth::login_and_save_credentials,
            auth::restore_login_session,
            auth::get_current_user_info,
            auth::logout_user,
            auth::logout_and_clear,
            auth::is_logged_in,
            // 课表命令
            schedule::import_schedule_from_saved_login,
            schedule::import_schedule_with_auto_relogin,
            schedule::update_schedule_with_diff,
            schedule::login_and_get_schedule,
            schedule::refresh_schedule,
            schedule::load_cached_schedule,
            schedule::save_schedule_cache,
            schedule::list_schedules,
            schedule::delete_schedule,
            schedule::switch_schedule,
            schedule::reorder_schedules,
            schedule::rename_schedule,
            schedule::update_schedule_info,
            // 导入导出命令
            import_export::export_schedule,
            import_export::import_schedule,
            // 考试命令
            exam::fetch_and_import_exams,
            // 配置命令
            config::save_time_table,
            config::delete_time_table,
            config::list_time_tables,
            config::apply_settings_to_all,
            config::get_app_config,
            config::save_app_config,
            config::clear_all_data,
            // 日期命令
            date::get_current_week,
            date::calculate_date,
            // 背景图命令
            background::save_background_image,
            background::delete_background_image,
            // 更新命令
            update::check_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
