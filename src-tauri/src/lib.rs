use log::error;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use base64;
use log::{debug, info};
use std::sync::OnceLock;
use tauri::image::Image;
use tauri::Emitter;
use tauri::Manager;
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_http::reqwest;

// 导入表情包服务模块
mod meme_server;
use meme_server::{MemeItem, MemeServerClient, MemeServerConfig};

// 导入配置管理器
mod config_manager;
use config_manager::{ConfigManager, ShortcutConfigs, UserPreferences};

// 导入系统托盘模块
mod sys_tray;
use sys_tray::create_system_tray;

// 创建一个全局静态HTTP客户端，确保只初始化一次
static MEME_CLIENT: OnceLock<MemeServerClient> = OnceLock::new();
// 创建一个全局静态配置管理器
static CONFIG_MANAGER: OnceLock<ConfigManager> = OnceLock::new();

fn get_meme_client() -> &'static MemeServerClient {
    MEME_CLIENT.get_or_init(|| {
        // 在实际应用中，可能需要从配置文件读取这些值
        let config = MemeServerConfig {
            api_url: "https://api.zvv.quest/search".to_string(),
            timeout_seconds: 10,
        };
        MemeServerClient::new(Some(config))
    })
}

fn get_config_manager() -> &'static ConfigManager {
    CONFIG_MANAGER.get_or_init(|| {
        ConfigManager::new("MemeMeow").expect("Failed to initialize config manager")
    })
}

// 原有的问候函数，可以保留用于测试
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 表情包搜索Tauri命令
#[tauri::command]
async fn search_memes(keyword: String) -> Result<Vec<MemeItem>, String> {
    info!("收到表情包搜索请求，关键词: {}", keyword);

    if keyword.trim().is_empty() {
        return Ok(Vec::new()); // 空关键词返回空结果
    }

    // 调用表情包服务客户端执行搜索
    match get_meme_client().search_memes(&keyword).await {
        Ok(memes) => {
            debug!("成功获取{}个表情包", memes.len());
            Ok(memes)
        }
        Err(err) => {
            debug!("获取表情包失败: {}", err);
            Err(err.to_string())
        }
    }
}

// 获取用户偏好设置
#[tauri::command]
fn get_user_preferences() -> Result<UserPreferences, String> {
    match get_config_manager().get_preferences() {
        Ok(prefs) => Ok(prefs),
        Err(err) => Err(err.to_string()),
    }
}

// 设置剪贴板复制选项
#[tauri::command]
fn set_copy_to_clipboard(enabled: bool) -> Result<(), String> {
    debug!("设置剪贴板复制选项: {}", enabled);
    match get_config_manager().update_clipboard_setting(enabled) {
        Ok(_) => Ok(()),
        Err(err) => Err(err.to_string()),
    }
}

// 获取快捷键配置
#[tauri::command]
fn get_shortcuts() -> Result<ShortcutConfigs, String> {
    match get_config_manager().get_shortcuts() {
        Ok(shortcuts) => Ok(shortcuts),
        Err(err) => Err(err.to_string()),
    }
}

// 设置快捷键配置
#[tauri::command]
fn set_shortcuts(shortcuts: ShortcutConfigs) -> Result<(), String> {
    debug!("设置快捷键配置: {:?}", shortcuts);
    match get_config_manager().update_shortcuts(shortcuts.clone()) {
        Ok(_) => {
            info!("快捷键配置已更新");
            Ok(())
        }
        Err(err) => {
            error!("更新快捷键配置失败: {}", err);
            Err(err.to_string())
        }
    }
}

// 剪贴板功能
#[tauri::command]
async fn copy_image_to_clipboard(image_url: String, window: tauri::Window) -> Result<(), String> {
    info!("Copying image to clipboard: {}", image_url);

    // 检查功能是否启用
    let copy_enabled = match get_config_manager().get_preferences() {
        Ok(prefs) => prefs.copy_to_clipboard,
        Err(err) => {
            error!("获取偏好设置失败: {}", err);
            return Err(format!("Failed to get preferences: {}", err));
        }
    };

    if !copy_enabled {
        info!("Clipboard copy is disabled in preferences");
        return Ok(());
    }

    // 下载图片数据
    let response = reqwest::get(&image_url).await.map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;

    // 将图片数据编码为 base64
    // let base64_image = base64::encode(&bytes);

    let image = Image::from_bytes(&bytes).unwrap();

    // 获取剪贴板管理器
    let clipboard = window.clipboard();

    clipboard.write_image(&image).map_err(|e| e.to_string())?;
    debug!("Image copied to clipboard successfully");
    // // 发送事件让前端处理剪贴板操作
    // // if let Err(e) = window.emit("copy-image-to-clipboard", image_url) {
    // //     return Err(format!("Failed to emit clipboard event: {}", e));
    // // }
    // // 将图片数据写入剪贴板
    // let mut clipboard = window.clipboard();
    // // clipboard.write_image(bytes.to_vec()).map_err(|e| e.to_string())?;
    // clipboard.write_image(bytes.to_vec()).map_err(|e| e.to_string())?;

    Ok(())
}

// 修改 run 函数以使用配置的快捷键并添加系统托盘
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin({
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{
                    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
                };

                tauri_plugin_global_shortcut::Builder::new()
                    .with_handler(move |app, shortcut, event| {
                        if event.state == ShortcutState::Pressed {
                            // 获取当前配置的切换应用快捷键
                            if let Some(config_manager) = CONFIG_MANAGER.get() {
                                match config_manager.get_toggle_app_shortcut() {
                                    Ok((mods, code)) => {
                                        let config_shortcut = Shortcut::new(Some(mods), code);

                                        // 检查是否匹配配置的快捷键
                                        if shortcut == &config_shortcut {
                                            info!("触发切换应用快捷键: {:?}", shortcut);
                                            let window = app.get_webview_window("main");
                                            if let Some(window) = window {
                                                match window.is_visible() {
                                                    Ok(is_visible) => {
                                                        if is_visible {
                                                            if let Err(e) = window.hide() {
                                                                error!("无法隐藏窗口: {}", e);
                                                            }
                                                        } else {
                                                            if let Err(e) = window.show() {
                                                                error!("无法显示窗口: {}", e);
                                                            } else if let Err(e) =
                                                                window.set_focus()
                                                            {
                                                                error!("无法设置窗口焦点: {}", e);
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        error!("无法获取窗口可见状态: {}", e);
                                                    }
                                                }
                                            } else {
                                                error!("无法获取主窗口引用");
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("无法获取切换应用快捷键配置: {}", e);
                                    }
                                }
                            } else {
                                error!("配置管理器未初始化");
                            }
                        }
                    })
                    .build()
            }
            #[cfg(not(desktop))]
            {
                // 非桌面平台的无操作插件
                tauri_plugin_global_shortcut::Builder::new().build()
            }
        })
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

                // 初始化配置管理器
                let config_manager = get_config_manager();

                // 注册快捷键
                register_app_shortcuts(app, config_manager);

                // 初始化系统托盘
                if let Err(e) = create_system_tray(app) {
                    error!("创建系统托盘失败: {}", e);
                } else {
                    info!("系统托盘创建成功");
                }
            }
            Ok(())
        })
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .setup(|app| {
            // 获取自动启动管理器
            let autostart_manager = app.autolaunch();
            // 启用 autostart
            let _ = autostart_manager.enable();
            // 检查 enable 状态
            println!(
                "registered for autostart? {}",
                autostart_manager.is_enabled().unwrap()
            );
            // 禁用 autostart
            let _ = autostart_manager.disable();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            search_memes,
            get_user_preferences,
            set_copy_to_clipboard,
            copy_image_to_clipboard,
            get_shortcuts,
            set_shortcuts,
            refresh_shortcuts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// 注册应用快捷键
#[cfg(desktop)]
fn register_app_shortcuts(app: &tauri::App, config_manager: &ConfigManager) {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

    match config_manager.get_toggle_app_shortcut() {
        Ok((mods, code)) => {
            let toggle_shortcut = Shortcut::new(Some(mods), code);

            match app.global_shortcut().register(toggle_shortcut) {
                Ok(_) => {
                    info!("成功注册应用切换快捷键");
                }
                Err(e) => {
                    // 注册失败，通知前端
                    error!("注册快捷键失败: {}", e);

                    // 发送事件通知前端注册失败
                    let main_window = app.get_webview_window("main");
                    if let Some(window) = main_window {
                        let _ = window.emit("shortcut-registration-failed", 
                            format!("无法注册快捷键 {}，可能与系统快捷键冲突。请在设置中配置其他快捷键。", 
                                format_shortcut_for_display(&mods, &code)));
                    }
                }
            }
        }
        Err(e) => {
            error!("获取快捷键配置失败: {}", e);

            // 通知用户配置获取失败
            let main_window = app.get_webview_window("main");
            if let Some(window) = main_window {
                let _ = window.emit(
                    "shortcut-registration-failed",
                    format!("无法获取快捷键配置: {}，请在设置中重新配置快捷键。", e),
                );
            }
        }
    }
}

// 格式化快捷键显示
#[cfg(desktop)]
fn format_shortcut_for_display(
    mods: &tauri_plugin_global_shortcut::Modifiers,
    code: &tauri_plugin_global_shortcut::Code,
) -> String {
    use tauri_plugin_global_shortcut::Modifiers;

    let mut parts = Vec::new();
    if mods.contains(Modifiers::CONTROL) {
        parts.push("Ctrl".to_string());
    }
    if mods.contains(Modifiers::ALT) {
        parts.push("Alt".to_string());
    }
    if mods.contains(Modifiers::SHIFT) {
        parts.push("Shift".to_string());
    }
    if mods.contains(Modifiers::META) {
        parts.push("Meta".to_string());
    }

    // 将Code转换为字符串
    let key = format!("{:?}", code);
    // 去除Code::前缀
    let key = key.replace("Code::", "").replace("Key", "");

    parts.push(key);
    parts.join("+")
}

// 添加刷新快捷键的命令
#[tauri::command]
fn refresh_shortcuts(app: tauri::AppHandle) -> Result<(), String> {
    #[cfg(desktop)]
    {
        use tauri_plugin_global_shortcut::GlobalShortcutExt;

        // 先清空当前注册的所有快捷键
        app.global_shortcut().unregister_all();

        // 从配置中重新注册快捷键
        if let Some(config_manager) = CONFIG_MANAGER.get() {
            match config_manager.get_toggle_app_shortcut() {
                Ok((mods, code)) => {
                    use tauri_plugin_global_shortcut::Shortcut;
                    let toggle_shortcut = Shortcut::new(Some(mods), code);

                    match app.global_shortcut().register(toggle_shortcut) {
                        Ok(_) => {
                            info!("成功刷新并注册应用切换快捷键");
                            Ok(())
                        }
                        Err(e) => {
                            error!("刷新注册快捷键失败: {}", e);

                            // 通知用户注册失败
                            let main_window = app.get_webview_window("main");
                            if let Some(window) = main_window {
                                let _ = window.emit(
                                    "shortcut-registration-failed",
                                    format!(
                                        "无法注册快捷键 {}，请在设置中配置其他快捷键。",
                                        format_shortcut_for_display(&mods, &code)
                                    ),
                                );
                            }

                            Err(format!("无法注册快捷键: {}", e))
                        }
                    }
                }
                Err(e) => {
                    error!("获取快捷键配置失败: {}", e);
                    Err(format!("无法获取快捷键配置: {}", e))
                }
            }
        } else {
            Err("配置管理器未初始化".into())
        }
    }

    #[cfg(not(desktop))]
    {
        Ok(())
    }
}
