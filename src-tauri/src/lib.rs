mod models;
mod crypto;
mod client;
mod storage;

use client::EduSystemState;
use models::{AppConfig, CachedSchedule, Course, ScheduleMetadata, UserCredentials, TimeTable, UserInfo};
use storage::StorageManager;
use chrono::{Utc, Duration, Datelike};
use std::fs;
use std::collections::HashSet;
use tauri::{Manager, Emitter};
use std::sync::Arc;

// ============== Tauri Commands ==============

/// 登录并获取用户信息（不获取课表）
#[tauri::command]
async fn login_and_get_user_info(
    state: tauri::State<'_, Arc<EduSystemState>>,
    username: String,
    password: String,
) -> Result<UserInfo, String> {
    let credentials = UserCredentials {
        username: username.clone(),
        password,
    };

    // 初始化全局 client
    state.initialize_client(username.clone());
    let mut client = state.get_client()?;

    // 执行登录
    let login_response = client.login(&credentials).await?;
    if !login_response.success {
        // 登录失败，清除状态
        state.logout();
        return Err(format!("登录失败: {}", login_response.message));
    }

    // 获取用户信息
    let user_info = client.get_user_info().await?;

    Ok(user_info)
}

/// 退出登录（旧版本兼容）
#[tauri::command]
fn logout_user(state: tauri::State<'_, Arc<EduSystemState>>) -> Result<(), String> {
    // 清除全局客户端状态（自动清除 cookies）
    state.logout();
    println!("已退出登录");
    Ok(())
}

/// ============================================================
/// 持久化登录相关命令
/// ============================================================

/// 登录并保存凭证
#[tauri::command]
async fn login_and_save_credentials(
    state: tauri::State<'_, Arc<EduSystemState>>,
    username: String,
    password: String,
) -> Result<crate::models::UserInfo, String> {
    let credentials = crate::models::UserCredentials {
        username: username.clone(),
        password,
    };

    // 初始化客户端并登录
    state.initialize_client(username.clone());
    let mut client = state.get_client()?;

    let login_response = client.login(&credentials).await?;
    if !login_response.success {
        state.logout();
        return Err(format!("登录失败: {}", login_response.message));
    }

    // 获取用户信息
    let user_info = client.get_user_info().await?;

    // 加密密码
    use crate::crypto::encrypt_password_dpapi;
    let encrypted_password = encrypt_password_dpapi(&credentials.password)
        .map_err(|e| format!("密码加密失败: {}", e))?;

    // 保存凭证
    let storage = StorageManager::new()?;
    let persistent_creds = crate::models::PersistentCredentials {
        username,
        password_encrypted: encrypted_password,
        edu_system_url: "https://xuanke.cufe.edu.cn/jwglxt/".to_string(), // CUFE 教务系统 URL
        saved_at: chrono::Utc::now().timestamp(),
    };

    storage.save_credentials(&persistent_creds)
        .map_err(|e| format!("保存凭证失败: {}", e))?;

    println!("凭证已保存到本地");
    Ok(user_info)
}

/// 恢复登录会话（应用启动时调用）
#[tauri::command]
async fn restore_login_session(
    state: tauri::State<'_, Arc<EduSystemState>>,
) -> Result<crate::models::UserInfo, String> {
    let storage = StorageManager::new()?;

    // 加载保存的凭证
    let creds = storage.load_credentials()?;

    // 解密密码
    use crate::crypto::decrypt_password_dpapi;
    let password = decrypt_password_dpapi(&creds.password_encrypted)
        .map_err(|e| format!("密码解密失败: {}", e))?;

    // 重新登录
    state.initialize_client(creds.username.clone());
    let mut client = state.get_client()?;

    let credentials = crate::models::UserCredentials {
        username: creds.username,
        password,
    };

    let login_response = client.login(&credentials).await?;
    if !login_response.success {
        state.logout();
        storage.clear_credentials()?; // 清除无效凭证
        return Err("保存的凭证已失效，请重新登录".to_string());
    }

    // 获取用户信息
    let user_info = client.get_user_info().await?;

    println!("已自动恢复登录状态");
    Ok(user_info)
}

/// 获取当前登录用户信息（不重新登录）
#[tauri::command]
async fn get_current_user_info(
    state: tauri::State<'_, Arc<EduSystemState>>,
) -> Result<Option<crate::models::UserInfo>, String> {
    // 检查是否已登录
    if !state.is_logged_in() {
        return Ok(None);
    }

    // 获取全局 client（复用登录时的会话）
    let client = state.get_client()?;

    // 获取用户信息（复用现有会话，不需要重新登录）
    let user_info = client.get_user_info().await?;
    Ok(Some(user_info))
}

/// 导入课表（带自动重新登录）
#[tauri::command]
async fn import_schedule_with_auto_relogin(
    state: tauri::State<'_, Arc<EduSystemState>>,
    year: i32,
    term: i32,
    schedule_name: String,
) -> Result<String, String> {
    // 尝试使用当前会话导入
    let import_result = import_schedule_from_saved_login(
        state.clone(),
        year,
        term,
        schedule_name.clone(),
    ).await;

    // 如果成功，直接返回
    if import_result.is_ok() {
        return import_result;
    }

    // 如果失败，尝试自动重新登录
    let storage = StorageManager::new()?;

    // 检查是否有保存的凭证
    let creds = match storage.load_credentials() {
        Ok(c) => c,
        Err(_) => return Err(import_result.unwrap_err()),
    };

    // 解密密码并重新登录
    use crate::crypto::decrypt_password_dpapi;
    let password = decrypt_password_dpapi(&creds.password_encrypted)
        .map_err(|e| format!("密码解密失败: {}", e))?;

    let credentials = crate::models::UserCredentials {
        username: creds.username,
        password,
    };

    state.auto_relogin(&credentials).await?;

    println!("已自动重新登录，重试导入课表");

    // 重试导入
    import_schedule_from_saved_login(
        state,
        year,
        term,
        schedule_name,
    ).await
}

/// 退出登录并清除所有凭证
#[tauri::command]
async fn logout_and_clear(
    state: tauri::State<'_, Arc<EduSystemState>>,
) -> Result<(), String> {
    // 清除内存状态
    state.logout();

    // 清除保存的凭证
    let storage = StorageManager::new()?;
    storage.clear_credentials()?;

    println!("已退出登录并清除所有凭证");
    Ok(())
}

/// 检查是否已登录
#[tauri::command]
fn is_logged_in(state: tauri::State<'_, Arc<EduSystemState>>) -> bool {
    state.is_logged_in()
}

/// 使用已保存的登录状态导入课表
#[tauri::command]
async fn import_schedule_from_saved_login(
    state: tauri::State<'_, Arc<EduSystemState>>,
    year: i32,
    term: i32,
    schedule_name: String,
) -> Result<String, String> {
    // 检查是否已登录
    if !state.is_logged_in() {
        return Err("未找到登录信息，请先在个人中心登录".to_string());
    }

    // 获取全局 client（复用登录时的会话）
    let client = state.get_client()?;

    // 获取课表
    let courses = client.get_schedule(year, term).await?;

    if courses.is_empty() {
        return Err("该学期暂无课程".to_string());
    }

    let storage = StorageManager::new()?;

    // 生成课表 ID
    let schedule_id = StorageManager::generate_schedule_id();

    // 保存课表
    let now = Utc::now().timestamp();
    let expire_time = now + (30 * 24 * 60 * 60); // 30 天后过期

    // 计算 sort_index (放在最后)
    let current_list = storage.list_schedules()?;
    let max_index = current_list.iter().filter_map(|s| s.sort_index).max().unwrap_or(-1);
    let sort_index = max_index + 1;

    let cached = CachedSchedule {
        id: schedule_id.clone(),
        name: schedule_name,
        courses,
        timestamp: now,
        expire_time,
        first_day: None,
        max_periods: None,
        weeks_count: None,
        time_table_id: None,
        sort_index: Some(sort_index),
    };

    storage.save_schedule(&cached)?;

    // 更新当前选中的课表 ID
    let mut config = storage.load_config().unwrap_or_default();
    config.current_schedule_id = Some(schedule_id.clone());
    storage.save_config(&config)?;

    Ok(schedule_id)
}


/// 登录并获取课表
#[tauri::command]
async fn login_and_get_schedule(
    state: tauri::State<'_, Arc<EduSystemState>>,
    username: String,
    password: String,
) -> Result<Vec<Course>, String> {
    let credentials = UserCredentials {
        username: username.clone(),
        password,
    };

    // 初始化全局 client
    state.initialize_client(username.clone());
    let mut client = state.get_client()?;

    // 执行登录
    let login_response = client.login(&credentials).await?;
    if !login_response.success {
        // 登录失败，清除状态
        state.logout();
        return Err(format!("登录失败: {}", login_response.message));
    }

    // 尝试多个学期：当前学期，如果失败则尝试上一学期，再失败尝试下一学期
    let now = Utc::now();
    let month = now.month();
    let year = now.year();

    // 确定当前推测的学年和学期
    let (current_year, current_term) = if month >= 9 {
        (year, 1) // 9月-12月: 第一学期 (如 2025-2026-1)
    } else if month == 1 {
        (year - 1, 1) // 1月: 仍视为第一学期 (如 2025-2026-1)
    } else {
        (year - 1, 2) // 2月-8月: 第二学期 (如 2025-2026-2)
    };

    // 生成待尝试的 (year, term) 列表
    let mut tasks = vec![
        (current_year, current_term),
    ];

    // 如果是第二学期，失败后尝试第一学期
    if current_term == 2 {
        tasks.push((current_year, 1));
    } else {
        // 如果是第一学期，失败后尝试上一学年的第二学期
        tasks.push((current_year - 1, 2));
    }

    // 再尝试一下未来/过去的一个备选 (例如当前是第一学期，也试试第二学期以防万一)
    if current_term == 1 {
        tasks.push((current_year, 2)); // 同一学年的第二学期
    } else {
        tasks.push((current_year + 1, 1)); // 下一学年的第一学期
    }

    let mut final_courses = Vec::new();
    let mut last_error = "未找到有效的课表数据".to_string();

    for (y, t) in tasks {
        println!("尝试获取课表: {}-学期{}", y, t);
        match client.get_schedule(y, t).await {
            Ok(courses) => {
                if !courses.is_empty() {
                    println!("成功获取课表: {} 门课程", courses.len());
                    final_courses = courses;
                    break;
                } else {
                    println!("课表为空，尝试下一个学期...");
                    last_error = "当前学期暂无课程".to_string();
                }
            },
            Err(e) => {
                println!("获取失败: {}, 尝试下一个学期...", e);
                last_error = e;
            }
        }
    }

    if final_courses.is_empty() {
        // 清除登录状态
        state.logout();
        return Err(format!("无法获取课表: {}", last_error));
    }

    Ok(final_courses)
}

/// 刷新课表数据（使用全局 client）
#[tauri::command]
async fn refresh_schedule(state: tauri::State<'_, Arc<EduSystemState>>) -> Result<Vec<Course>, String> {
    // 检查是否已登录
    if !state.is_logged_in() {
        return Err("未找到登录信息，请先在个人中心登录".to_string());
    }

    // 获取全局 client（复用登录时的会话）
    let client = state.get_client()?;

    // 计算当前学期
    let now = Utc::now();
    let month = now.month();
    let year = now.year();

    let (school_year, term) = if month >= 9 {
        (year, 1)
    } else if month == 1 {
        (year - 1, 1)
    } else {
        (year - 1, 2)
    };

    // 获取课表
    let courses = client.get_schedule(school_year, term).await?;

    Ok(courses)
}

/// 加载缓存的课表数据
#[tauri::command]
fn load_cached_schedule(schedule_id: Option<String>) -> Result<Vec<Course>, String> {
    let storage = StorageManager::new()?;

    // 如果没有指定 ID,尝试从配置中获取
    let id = if let Some(sid) = schedule_id {
        sid
    } else {
        let config = storage.load_config()?;
        if let Some(current_id) = config.current_schedule_id {
            current_id
        } else {
            return Err("没有选中的课表".to_string());
        }
    };

    let cached = storage.load_schedule(&id)?;

    if cached.is_expired() {
        return Err("课表数据已过期，请刷新".to_string());
    }

    Ok(cached.courses)
}

/// 保存课表数据到缓存
#[tauri::command]
fn save_schedule_cache(
    courses: Vec<Course>, 
    name: String, 
    schedule_id: Option<String>, 
    first_day: Option<i64>,
    max_periods: Option<i32>,
    weeks_count: Option<i32>,
    time_table_id: Option<String>
) -> Result<String, String> {
    let storage = StorageManager::new()?;

    let now = Utc::now().timestamp();
    let expire_time = now + (30 * 24 * 60 * 60); // 30 天后过期

    // 生成或使用指定的 ID
    let id = if let Some(sid) = schedule_id {
        sid
    } else {
        StorageManager::generate_schedule_id()
    };

    // 如果是新课表，需要计算 sort_index
    let mut sort_index = None;
    if let Ok(existing) = storage.load_schedule(&id) {
        sort_index = existing.sort_index;
    } 
    if sort_index.is_none() {
        let current_list = storage.list_schedules()?;
        let max_index = current_list.iter().filter_map(|s| s.sort_index).max().unwrap_or(-1);
        sort_index = Some(max_index + 1);
    }

    let cached = CachedSchedule {
        id: id.clone(),
        name,
        courses,
        timestamp: now,
        expire_time,
        first_day,
        max_periods,
        weeks_count,
        time_table_id,
        sort_index,
    };

    storage.save_schedule(&cached)?;

    // 更新当前选中的课表 ID
    let mut config = storage.load_config().unwrap_or_default();
    config.current_schedule_id = Some(id.clone());

    // 如果提供了 first_day，也更新全局配置 (兼容性)
    if let Some(fd) = first_day {
        config.first_day = Some(fd);
    }

    storage.save_config(&config)?;

    Ok(id)
}

/// 获取所有课表列表
#[tauri::command]
fn list_schedules() -> Result<Vec<ScheduleMetadata>, String> {
    let storage = StorageManager::new()?;
    storage.list_schedules()
}

/// 删除指定课表
#[tauri::command]
fn delete_schedule(schedule_id: String) -> Result<(), String> {
    let storage = StorageManager::new()?;

    // 获取当前配置
    let mut config = storage.load_config().unwrap_or_default();

    // 如果删除的是当前选中的课表,清空选中状态
    if let Some(current_id) = &config.current_schedule_id {
        if current_id == &schedule_id {
            config.current_schedule_id = None;
            storage.save_config(&config)?;
        }
    }

    storage.delete_schedule(&schedule_id)
}

/// 切换当前课表
#[tauri::command]
fn switch_schedule(schedule_id: String) -> Result<(), String> {
    let storage = StorageManager::new()?;
    let mut config = storage.load_config().unwrap_or_default();
    config.current_schedule_id = Some(schedule_id.clone());

    // 同步课表的 first_day 到全局配置
    let schedule = storage.load_schedule(&schedule_id)?;
    config.first_day = schedule.first_day;

    storage.save_config(&config)?;
    Ok(())
}

/// 重新排序课表
#[tauri::command]
fn reorder_schedules(sorted_ids: Vec<String>) -> Result<(), String> {
    let storage = StorageManager::new()?;

    for (index, id) in sorted_ids.iter().enumerate() {
        if let Ok(mut schedule) = storage.load_schedule(id) {
            schedule.sort_index = Some(index as i32);
            storage.save_schedule(&schedule)?;
        }
    }

    Ok(())
}

/// 更新课表信息
#[tauri::command]
fn update_schedule_info(
    schedule_id: String, 
    first_day: Option<i64>,
    max_periods: Option<i32>,
    weeks_count: Option<i32>,
    time_table_id: Option<String>
) -> Result<(), String> {
    println!("更新课表信息 - schedule_id: {}", schedule_id);

    let storage = StorageManager::new()?;

    // 加载课表
    let mut cached = storage.load_schedule(&schedule_id)?;
    
    // 更新字段
    cached.first_day = first_day;
    cached.max_periods = max_periods;
    cached.weeks_count = weeks_count;
    cached.time_table_id = time_table_id;

    // 保存
    storage.save_schedule(&cached)?;
    
    // 如果这是当前选中的课表，也更新全局配置 (兼容性)
    let config = storage.load_config()?;
    if let Some(ref current_id) = config.current_schedule_id {
        if current_id == &schedule_id {
            let mut new_config = config;
            new_config.first_day = first_day;
            if let Some(mp) = max_periods { new_config.max_periods = Some(mp); }
            storage.save_config(&new_config)?;
        }
    }

    Ok(())
}

/// 重命名课表
#[tauri::command]
fn rename_schedule(schedule_id: String, new_name: String) -> Result<(), String> {
    println!("重命名课表 - schedule_id: {}, new_name: {}", schedule_id, new_name);

    let storage = StorageManager::new()?;

    // 加载课表
    let mut cached = storage.load_schedule(&schedule_id)?;

    // 更新名称
    cached.name = new_name.clone();

    // 保存
    storage.save_schedule(&cached)?;

    // 如果这是当前选中的课表，需要更新元数据列表中的名称
    // 由于元数据列表是动态生成的，这里只需要保存更新后的课表即可
    // 前端会重新加载列表

    Ok(())
}

/// 获取当前周次
#[tauri::command]
fn get_current_week(first_day: Option<i64>) -> i32 {
    if let Some(first_day) = first_day {
        let now = Utc::now().timestamp();
        let weeks = ((now - first_day) / (7 * 24 * 60 * 60)) + 1;
        weeks.max(1) as i32
    } else {
        1
    }
}

/// 计算指定周次和星期的日期
/// 返回格式: "M/D" (如 "1/15")
#[tauri::command]
fn calculate_date(first_day: i64, target_week: i32, target_day: i32) -> String {
    let first_date = chrono::DateTime::<Utc>::from_timestamp(first_day, 0)
        .unwrap()
        .date_naive();

    // 计算目标日期
    let target_date = first_date
        + Duration::weeks((target_week - 1) as i64)
        + Duration::days((target_day - 1) as i64);

    format!("{}/{}", target_date.month(), target_date.day())
}

/// 保存背景图
#[tauri::command]
async fn save_background_image(source_path: String) -> Result<String, String> {
    let storage = StorageManager::new()?;
    let bg_dir = storage.background_dir();

    // 读取源文件
    let file_name = format!("bg_{}.jpg", Utc::now().timestamp());
    let dest_path = bg_dir.join(&file_name);

    fs::copy(&source_path, &dest_path)
        .map_err(|e| format!("复制背景图失败: {}", e))?;

    // 保存到配置
    let mut config = storage.load_config().unwrap_or_default();
    config.background_image = Some(file_name.clone());
    storage.save_config(&config)?;

    Ok(file_name)
}

/// 删除背景图
#[tauri::command]
fn delete_background_image() -> Result<(), String> {
    let storage = StorageManager::new()?;
    let config = storage.load_config()?;

    if let Some(filename) = config.background_image {
        storage.delete_background(&filename)?;
    }

    // 更新配置
    let mut config = storage.load_config().unwrap_or_default();
    config.background_image = None;
    storage.save_config(&config)?;

    Ok(())
}

/// 获取应用配置 (含自动迁移逻辑)
#[tauri::command]
fn get_app_config() -> Result<AppConfig, String> {
    let storage = StorageManager::new()?;
    let mut config = storage.load_config()?;

    // 自动迁移：如果 time_tables 为空但 period_times 存在，创建默认时间表
    let has_time_tables = config.time_tables.as_ref().map_or(false, |v| !v.is_empty());
    if !has_time_tables {
        if let Some(ref periods) = config.period_times {
            let default_table = TimeTable {
                id: "default".to_string(),
                name: "默认时间表".to_string(),
                periods: periods.clone(),
            };
            config.time_tables = Some(vec![default_table]);
            storage.save_config(&config)?;
        }
    }

    // 自动迁移：智能更新 edu_systems
    use crate::models::EduSystem;

    // 定义所有默认的教务系统
    let default_systems = vec![
        EduSystem {
            id: "cufe".to_string(),
            name: "中央财经大学".to_string(),
            url: "https://xuanke.cufe.edu.cn/jwglxt/".to_string(),
            parser_type: "cufe_default".to_string(),
            enabled: true,
        },
        EduSystem {
            id: "zju".to_string(),
            name: "浙江大学".to_string(),
            url: "https://zdbk.zju.edu.cn/jwglxt/".to_string(),
            parser_type: "zju_default".to_string(),
            enabled: true,
        },
    ];

    let mut needs_save = false;
    let mut current_systems = config.edu_systems.take().unwrap_or_default();

    // 构建现有系统的 ID 集合
    let existing_ids: HashSet<String> =
        current_systems.iter().map(|s| s.id.clone()).collect();

    // 添加缺失的默认系统
    for default_system in default_systems {
        if !existing_ids.contains(&default_system.id) {
            println!("添加新的教务系统: {}", default_system.name);
            current_systems.push(default_system);
            needs_save = true;
        }
    }

    // 如果有旧配置且包含 cufe URL，保留用户的 URL
    if let Some(ref old_url) = config.edu_system_url {
        if old_url.contains("cufe.edu.cn") {
            if let Some(cufe_system) = current_systems.iter_mut().find(|s| s.id == "cufe") {
                if cufe_system.url != *old_url {
                    cufe_system.url = old_url.clone();
                    needs_save = true;
                }
            }
        }
    }

    // 确保 last_edu_system_id 有值
    if config.last_edu_system_id.is_none() && !current_systems.is_empty() {
        config.last_edu_system_id = Some(current_systems[0].id.clone());
        needs_save = true;
    }

    config.edu_systems = Some(current_systems);

    if needs_save {
        storage.save_config(&config)?;
    }

    Ok(config)
}

/// 保存应用配置
#[tauri::command]
fn save_app_config(config: AppConfig) -> Result<(), String> {
    let storage = StorageManager::new()?;
    storage.save_config(&config)?;
    Ok(())
}

/// 保存时间表
#[tauri::command]
fn save_time_table(time_table: TimeTable) -> Result<(), String> {
    let storage = StorageManager::new()?;
    let mut config = storage.load_config().unwrap_or_default();

    let mut tables = config.time_tables.unwrap_or_default();
    
    // 如果ID已存在则更新，否则添加
    if let Some(index) = tables.iter().position(|t| t.id == time_table.id) {
        tables[index] = time_table;
    } else {
        tables.push(time_table);
    }
    
    config.time_tables = Some(tables);
    storage.save_config(&config)?;
    Ok(())
}

/// 删除时间表
#[tauri::command]
fn delete_time_table(id: String) -> Result<(), String> {
    let storage = StorageManager::new()?;
    let mut config = storage.load_config().unwrap_or_default();

    if let Some(tables) = config.time_tables {
        let new_tables: Vec<TimeTable> = tables.into_iter().filter(|t| t.id != id).collect();
        config.time_tables = Some(new_tables);
        storage.save_config(&config)?;
    }
    Ok(())
}

/// 获取时间表列表
#[tauri::command]
fn list_time_tables() -> Result<Vec<TimeTable>, String> {
    let config = get_app_config()?;
    Ok(config.time_tables.unwrap_or_default())
}

/// 将指定课表的设置应用到所有课表
#[tauri::command]
fn apply_settings_to_all(source_schedule_id: String) -> Result<(), String> {
    let storage = StorageManager::new()?;
    
    // 1. 获取源课表设置
    let source = storage.load_schedule(&source_schedule_id)?;
    // 注意：不应用 first_day，因为不同学期或不同用户的课表起始日可能不同
    let max_periods = source.max_periods;
    let weeks_count = source.weeks_count;
    let time_table_id = source.time_table_id;
    
    // 2. 获取所有课表ID
    let metadata_list = storage.list_schedules()?;
    
    // 3. 遍历更新
    for meta in metadata_list {
        if meta.id == source_schedule_id {
            continue;
        }
        
        let mut schedule = storage.load_schedule(&meta.id)?;
        // schedule.first_day = first_day; // SKIP
        schedule.max_periods = max_periods;
        schedule.weeks_count = weeks_count;
        schedule.time_table_id = time_table_id.clone();
        
        storage.save_schedule(&schedule)?;
    }
    
    // 4. 更新全局配置 (如果存在相关项)
    let mut config = storage.load_config()?;
    // config.first_day = first_day; // SKIP
    config.max_periods = max_periods;
    storage.save_config(&config)?;
    
    Ok(())
}

/// 清空所有数据
#[tauri::command]
fn clear_all_data() -> Result<(), String> {
    let storage = StorageManager::new()?;
    storage.clear_all()?;
    Ok(())
}

// ============== Main Entry Point ==============

mod parser; 

// ... existing code ...

/// 解析 HTML 课表
#[tauri::command]
fn parse_html_schedule(html: String, parser_type: Option<String>) -> Result<Vec<Course>, String> {
    let parser = parser_type.as_deref().unwrap_or("cufe_default");
    parser::parse_html_with_parser(&html, parser)
}

/// 从浏览器导入课表
#[tauri::command]
async fn import_from_browser(app: tauri::AppHandle, html: String, name: Option<String>, parser_type: Option<String>) -> Result<String, String> {
    println!("=== 浏览器导入被调用 ===");
    println!("HTML 长度: {} 字符", html.len());
    println!("名称参数: {:?}", name);
    println!("解析器类型: {:?}", parser_type);

    // 解析 HTML
    let parser = parser_type.as_deref().unwrap_or("cufe_default");
    let courses = parser::parse_html_with_parser(&html, parser)?;
    println!("解析到 {} 门课程", courses.len());

    // 生成课表ID
    let schedule_id = StorageManager::generate_schedule_id();

    // 使用提供的名称或默认名称
    let schedule_name = name.unwrap_or_else(|| {
        format!("从浏览器导入 {}", chrono::Utc::now().format("%Y-%m-%d"))
    });

    println!("课表名称: {}", schedule_name);

    // 保存课表
    let storage = StorageManager::new()?;
    let now = Utc::now().timestamp();
    let expire_time = now + (30 * 24 * 60 * 60); // 30 天后过期

    // 计算 sort_index (放在最后)
    let current_list = storage.list_schedules()?;
    let max_index = current_list.iter().filter_map(|s| s.sort_index).max().unwrap_or(-1);
    let sort_index = max_index + 1;

    let cached = CachedSchedule {
        id: schedule_id.clone(),
        name: schedule_name,
        courses,
        timestamp: now,
        expire_time,
        first_day: None, 
        max_periods: None,
        weeks_count: None,
        time_table_id: None,
        sort_index: Some(sort_index),
    };

    storage.save_schedule(&cached)?;
    println!("课表已保存到文件");

    // 更新当前选中的课表 ID
    let mut config = storage.load_config().unwrap_or_default();
    config.current_schedule_id = Some(schedule_id.clone());
    storage.save_config(&config)?;
    println!("全局配置已更新");

    // 通过事件通知前端，导入成功并返回课表ID和课程数量
    app.emit("schedule-imported", serde_json::json!({
        "schedule_id": schedule_id,
        "course_count": cached.courses.len(),
        "schedule_name": cached.name
    }))
        .map_err(|e| format!("发送事件失败: {}", e))?;

    println!("=== 浏览器导入完成 ===");
    Ok(schedule_id)
}

// ============== Main Entry Point ==============

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // CUFE 教务系统 URL
    let base_url = "https://xuanke.cufe.edu.cn/jwglxt".to_string();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Arc::new(EduSystemState::new(base_url)))
        .invoke_handler(tauri::generate_handler![
            // 用户相关
            login_and_get_user_info,
            login_and_save_credentials,
            restore_login_session,
            get_current_user_info,
            logout_user,
            logout_and_clear,
            is_logged_in,
            import_schedule_from_saved_login,
            import_schedule_with_auto_relogin,
            // 登录相关
            login_and_get_schedule,
            refresh_schedule,
            load_cached_schedule,
            save_schedule_cache,
            // 课表管理
            list_schedules,
            delete_schedule,
            switch_schedule,
            reorder_schedules,
            rename_schedule,
            update_schedule_info,
            save_time_table,
            delete_time_table,
            list_time_tables,
            apply_settings_to_all,
            // 数据相关
            get_current_week,
            calculate_date,
            parse_html_schedule,
            import_from_browser,
            // 配置相关
            save_background_image,
            delete_background_image,
            get_app_config,
            save_app_config,
            clear_all_data,
            open_login_window,
            get_window_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 检查窗口是否存在
#[tauri::command]
fn get_window_state(app: tauri::AppHandle, window_label: String) -> Result<String, String> {
    if let Some(_window) = app.get_webview_window(&window_label) {
        Ok("exists".to_string())
    } else {
        Err("Window not found".to_string())
    }
}

/// 打开登录窗口
#[tauri::command]
async fn open_login_window(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri::{WebviewUrl, WebviewWindowBuilder};

    let win_label = "login_window";
    
    // 如果窗口已存在，聚焦
    if let Some(win) = app.get_webview_window(win_label) {
        win.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }

    let script = r#"
        (function() {
            // Override window.open to force same-window navigation
            // Many edu systems use window.open for sub-modules
            window.open = function(url) {
                if (url) {
                    // Handle relative URLs
                    window.location.href = url;
                }
                return window;
            };

            // Fix links to open in same window (Baidu defaults to _blank)
            function fixLinks() {
                try {
                    const links = document.querySelectorAll('a[target="_blank"]');
                    links.forEach(link => {
                        try { link.target = '_self'; } catch(e) {}
                    });
                     // Also fix common frame layouts if accessible
                    try {
                        const frames = document.querySelectorAll('iframe');
                        frames.forEach(frame => {
                            try {
                                const fDoc = frame.contentDocument;
                                if (fDoc) {
                                     const fLinks = fDoc.querySelectorAll('a[target="_blank"]');
                                     fLinks.forEach(l => l.target = '_self');
                                }
                            } catch(e) {}
                        });
                    } catch(e) {}
                } catch(e) {}
            }

            // Run frequently to catch dynamic content
            setInterval(fixLinks, 1000);
            window.addEventListener('DOMContentLoaded', fixLinks);
            window.addEventListener('load', fixLinks);
            document.addEventListener('click', function(e) {
                // aggressively fix target if clicking an anchor
                let target = e.target;
                while (target && target.tagName !== 'A') {
                    target = target.parentElement;
                }
                if (target && target.tagName === 'A') {
                    target.target = '_self';
                }
            }, true);

            // Function to add the button
            function addImportBtn() {
                if (document.getElementById('tauri-import-btn')) return;

                const btn = document.createElement('button');
                btn.id = 'tauri-import-btn';
                btn.innerHTML = '📥 导入当前课表';
                btn.style.cssText = `
                    position: fixed;
                    bottom: 30px;
                    right: 30px;
                    z-index: 2147483647;
                    padding: 12px 24px;
                    background-color: #409EFF;
                    color: white;
                    border: none;
                    border-radius: 8px;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.2);
                    font-size: 16px;
                    font-weight: bold;
                    cursor: pointer;
                    transition: all 0.3s;
                `;

                btn.onmouseover = () => btn.style.transform = 'scale(1.05)';
                btn.onmouseout = () => btn.style.transform = 'scale(1)';

                btn.onclick = async function() {
                    try {
                        const html = document.documentElement.outerHTML;

                        btn.innerHTML = '⏳ 正在复制...';
                        btn.style.backgroundColor = '#E6A23C';

                        // 尝试使用 Clipboard API
                        if (navigator.clipboard && navigator.clipboard.writeText) {
                            await navigator.clipboard.writeText(html);
                            console.log('HTML 已复制到剪贴板');
                            btn.innerHTML = '✅ 已复制！请关闭此窗口';
                            btn.style.backgroundColor = '#67C23A';

                            // 禁用按钮，防止重复点击
                            btn.disabled = true;
                            btn.style.cursor = 'default';
                            btn.onclick = null;
                        } else {
                            // 降级方案：使用传统方法
                            const textArea = document.createElement('textarea');
                            textArea.value = html;
                            textArea.style.position = 'fixed';
                            textArea.style.left = '-999999px';
                            document.body.appendChild(textArea);
                            textArea.select();

                            try {
                                document.execCommand('copy');
                                console.log('HTML 已复制到剪贴板（传统方法）');
                                btn.innerHTML = '✅ 已复制！请关闭此窗口';
                                btn.style.backgroundColor = '#67C23A';
                                document.body.removeChild(textArea);

                                // 禁用按钮，防止重复点击
                                btn.disabled = true;
                                btn.style.cursor = 'default';
                                btn.onclick = null;
                            } catch (err) {
                                console.error('复制失败:', err);
                                btn.innerHTML = '❌ 复制失败';
                                btn.style.backgroundColor = '#F56C6C';
                                document.body.removeChild(textArea);

                                setTimeout(() => {
                                    btn.innerHTML = '📥 导入当前课表';
                                    btn.style.backgroundColor = '#409EFF';
                                }, 3000);
                            }
                        }
                    } catch (err) {
                        console.error('导入失败:', err);
                        btn.innerHTML = '❌ 复制失败';
                        btn.style.backgroundColor = '#F56C6C';

                        setTimeout(() => {
                            btn.innerHTML = '📥 导入当前课表';
                            btn.style.backgroundColor = '#409EFF';
                        }, 3000);
                    }
                };

                document.body.appendChild(btn);
            }

            // Observe DOM changes to ensure button persists
            const observer = new MutationObserver((mutations) => {
                if (!document.body) return;
                addImportBtn();
            });

            if (document.body) {
                addImportBtn();
                observer.observe(document.body, { childList: true, subtree: true });
            } else {
                window.addEventListener('DOMContentLoaded', () => {
                    addImportBtn();
                    observer.observe(document.body, { childList: true, subtree: true });
                });
            }
        })();
    "#;

    WebviewWindowBuilder::new(&app, win_label, WebviewUrl::External(url::Url::parse(&url).map_err(|e| e.to_string())?))
        .title("教务系统 - 登录并导入")
        .inner_size(1024.0, 768.0)
        .initialization_script(script)
        .build()
        .map_err(|e| e.to_string())?;

    Ok(())
}
