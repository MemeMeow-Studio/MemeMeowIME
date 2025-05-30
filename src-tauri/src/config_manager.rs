use log::{debug, error, info};
use serde::{de, Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri_plugin_global_shortcut::{Code, Modifiers};
use crate::utils::key_map::ShortcutConfig;
use crate::utils::misc::ApiUrl;

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            modifiers: vec!["ctrl".to_string(), "alt".to_string()],
            key: "v".to_string(),
            action: "切换应用窗口".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserPreferences {
    #[serde(default = "default_true")]
    pub copy_to_clipboard: bool,
    #[serde(default)]
    pub shortcuts: ShortcutConfigs,
    #[serde(default)]
    pub api_urls: ApiUrlConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ShortcutConfigs {
    #[serde(default = "default_toggle_app_shortcut")]
    pub toggle_app: ShortcutConfig,
    // 可以添加更多快捷键配置
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiUrlConfig {
    #[serde(default = "default_api_urls")]
    pub urls: Vec<ApiUrl>,
    #[serde(default = "default_active_api_index")]
    pub active_index: usize,
}




impl Default for ApiUrlConfig {
    fn default() -> Self {
        Self {
            urls: default_api_urls(),
            active_index: 0,
        }
    }
}



fn default_true() -> bool {
    true
}

fn default_toggle_app_shortcut() -> ShortcutConfig {
    ShortcutConfig {
        modifiers: vec!["ctrl".to_string(), "alt".to_string()],
        key: "n".to_string(),
        action: "切换应用窗口".to_string(),
    }
}

fn default_api_urls() -> Vec<ApiUrl> {
    vec![
        ApiUrl {
            name: "默认API".to_string(),
            url: "https://mememeow.morami.icu".to_string(),
        }
    ]
}

fn default_active_api_index() -> usize {
    0
}




impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            copy_to_clipboard: true,
            shortcuts: ShortcutConfigs::default(),
            api_urls: ApiUrlConfig::default(),
        }
    }
}

pub struct ConfigManager {
    path: PathBuf,
    preferences: Arc<Mutex<UserPreferences>>,
}

impl ConfigManager {
    pub fn new(app_name: &str) -> Result<Self, io::Error> {
        // 获取或创建配置目录
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(app_name);

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        debug!("配置目录: {:?}", config_dir);

        let config_path = config_dir.join("preferences.json");
        let preferences = match Self::load_preferences(&config_path) {
            Ok(prefs) => {
                info!("加载用户配置成功");
                prefs
            }
            Err(err) => {
                error!("加载用户配置失败: {}，将使用默认配置", err);
                UserPreferences::default()
            }
        };

        Ok(Self {
            path: config_path,
            preferences: Arc::new(Mutex::new(preferences)),
        })
    }

    // 加载偏好设置
    fn load_preferences(path: &PathBuf) -> Result<UserPreferences, io::Error> {
        if !path.exists() {
            debug!("配置文件不存在，将创建默认配置: {:?}", path);
            let default_prefs = UserPreferences::default();
            let mut file = File::create(path)?;
            let json = serde_json::to_string_pretty(&default_prefs)?;
            file.write_all(json.as_bytes())?;
            return Ok(default_prefs);
        }

        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        match serde_json::from_str(&contents) {
            Ok(prefs) => Ok(prefs),
            Err(err) => {
                error!("解析配置文件失败: {}", err);
                Err(io::Error::new(io::ErrorKind::InvalidData, err))
            }
        }
    }

    // 保存偏好设置
    fn save_preferences(&self) -> Result<(), io::Error> {
        let prefs = match self.preferences.try_lock() {
            Ok(guard) => guard.clone(),
            Err(err) => {
                error!("获取偏好锁失败: {}", err);
                return Err(io::Error::new(io::ErrorKind::Other, "获取偏好锁失败"));
            }
        };

        let json = serde_json::to_string_pretty(&prefs)?;
        let mut file = File::create(&self.path)?;
        file.write_all(json.as_bytes())?;
        debug!("配置已保存到: {:?}", self.path);
        Ok(())
    }

    fn save_preferences_locked(&self, prefs: &UserPreferences) -> Result<(), io::Error> {
        let json = serde_json::to_string_pretty(prefs)?;
        let mut file = File::create(&self.path)?;
        file.write_all(json.as_bytes())?;
        debug!("配置已保存到: {:?}", self.path);
        Ok(())
    }

    // 获取偏好设置
    pub fn get_preferences(&self) -> Result<UserPreferences, io::Error> {
        debug!("尝试获取偏好设置锁");
        match self.preferences.try_lock() {
            Ok(guard) => {
                debug!("成功获取偏好设置锁");
                Ok(guard.clone())
            }
            Err(_) => {
                error!("无法获取偏好设置锁，可能已被其他线程持有");
                Err(io::Error::new(io::ErrorKind::Other, "获取偏好锁失败"))
            }
        }
    }

    // 更新偏好设置
    pub fn update_preferences(&self, new_prefs: UserPreferences) -> Result<(), io::Error> {
        match self.preferences.try_lock() {
            Ok(mut guard) => {
                *guard = new_prefs;
                self.save_preferences()
            }
            Err(err) => {
                error!("获取偏好锁失败: {}", err);
                // 新增调试信息
                error!("可能发生死锁或锁被长时间占用");
                Err(io::Error::new(io::ErrorKind::Other, "获取偏好锁失败"))
            }
        }
    }

    // 更新剪贴板设置
    pub fn update_clipboard_setting(&self, enabled: bool) -> Result<(), io::Error> {
        debug!("尝试更新剪贴板设置 a");
        match self.preferences.try_lock() {
            Ok(mut guard) => {
                debug!("成功获取偏好设置锁 b");
                guard.copy_to_clipboard = enabled;
                debug!("剪贴板设置已更新: {}", enabled);
                self.save_preferences_locked(&guard.clone())
            }
            Err(err) => {
                error!("获取偏好锁失败: {}", err);
                Err(io::Error::new(io::ErrorKind::Other, "获取偏好锁失败"))
            }
        }
    }

    // 更新快捷键设置
    pub fn update_shortcuts(&self, shortcuts: ShortcutConfigs) -> Result<(), io::Error> {
        match self.preferences.lock() {
            Ok(mut guard) => {
                guard.shortcuts = shortcuts;
                self.save_preferences_locked(&guard.clone())
            }
            Err(err) => {
                error!("获取偏好锁失败: {}", err);
                Err(io::Error::new(io::ErrorKind::Other, "获取偏好锁失败"))
            }
        }
    }

    // 获取快捷键配置
    pub fn get_shortcuts(&self) -> Result<ShortcutConfigs, io::Error> {
        match self.preferences.lock() {
            Ok(guard) => Ok(guard.shortcuts.clone()),
            Err(err) => {
                error!("获取偏好锁失败: {}", err);
                Err(io::Error::new(io::ErrorKind::Other, "获取偏好锁失败"))
            }
        }
    }

    // 获取应用切换快捷键
    pub fn get_toggle_app_shortcut(&self) -> Result<(Modifiers, Code), io::Error> {
        match self.preferences.lock() {
            Ok(guard) => Ok(guard.shortcuts.toggle_app.to_tauri_shortcut()),
            Err(err) => {
                error!("获取偏好锁失败: {}", err);
                Err(io::Error::new(io::ErrorKind::Other, "获取偏好锁失败"))
            }
        }
    }

    // 获取当前活跃的API URL
    pub fn get_active_api_url(&self) -> Result<String, io::Error> {
        match self.preferences.try_lock() {
            Ok(guard) => {
                let config = &guard.api_urls;
                if config.urls.is_empty() {
                    return Ok("https://mememeow.morami.icu".to_string());
                }
                
                let index = if config.active_index < config.urls.len() {
                    config.active_index
                } else {
                    0
                };
                
                Ok(config.urls[index].url.clone())
            },
            Err(err) => {
                error!("获取偏好锁失败: {}", err);
                Err(io::Error::new(io::ErrorKind::Other, "获取偏好锁失败"))
            }
        }
    }

    // 获取API URL配置
    pub fn get_api_url_config(&self) -> Result<ApiUrlConfig, io::Error> {
        match self.preferences.try_lock() {
            Ok(guard) => Ok(guard.api_urls.clone()),
            Err(err) => {
                error!("获取偏好锁失败: {}", err);
                Err(io::Error::new(io::ErrorKind::Other, "获取偏好锁失败"))
            }
        }
    }

    // 更新API URL配置
    pub fn update_api_url_config(&self, config: ApiUrlConfig) -> Result<(), io::Error> {
        match self.preferences.try_lock() {
            Ok(mut guard) => {
                guard.api_urls = config;
                debug!("API URL配置已更新");
                self.save_preferences_locked(&guard.clone())
            },
            Err(err) => {
                error!("获取偏好锁失败: {}", err);
                Err(io::Error::new(io::ErrorKind::Other, "获取偏好锁失败"))
            }
        }
    }

    // 设置活跃的API URL
    pub fn set_active_api_url(&self, index: usize) -> Result<(), io::Error> {
        match self.preferences.try_lock() {
            Ok(mut guard) => {
                if index < guard.api_urls.urls.len() {
                    guard.api_urls.active_index = index;
                    debug!("活跃API URL已更新为索引 {}", index);
                    self.save_preferences_locked(&guard.clone())
                } else {
                    Err(io::Error::new(io::ErrorKind::InvalidInput, "API URL索引超出范围"))
                }
            },
            Err(err) => {
                error!("获取偏好锁失败: {}", err);
                Err(io::Error::new(io::ErrorKind::Other, "获取偏好锁失败"))
            }
        }
    }

    // 添加API URL
    pub fn add_api_url(&self, name: String, url: String) -> Result<(), io::Error> {
        match self.preferences.try_lock() {
            Ok(mut guard) => {
                guard.api_urls.urls.push(ApiUrl { name, url });
                debug!("已添加新的API URL");
                self.save_preferences_locked(&guard.clone())
            }
            Err(err) => {
                error!("获取偏好锁失败: {}", err);
                Err(io::Error::new(io::ErrorKind::Other, "获取偏好锁失败"))
            }
        }
    }

    // 删除API URL
    pub fn remove_api_url(&self, index: usize) -> Result<(), io::Error> {
        match self.preferences.try_lock() {
            Ok(mut guard) => {
                if index < guard.api_urls.urls.len() {
                    guard.api_urls.urls.remove(index);
                    
                    // 如果删除的是当前活跃的API，则将活跃索引重置为0
                    if guard.api_urls.active_index >= guard.api_urls.urls.len() {
                        guard.api_urls.active_index = 0;
                    }
                    
                    debug!("已删除API URL，索引: {}", index);
                    self.save_preferences_locked(&guard.clone())
                } else {
                    Err(io::Error::new(io::ErrorKind::InvalidInput, "API URL索引超出范围"))
                }
            },
            Err(err) => {
                error!("获取偏好锁失败: {}", err);
                Err(io::Error::new(io::ErrorKind::Other, "获取偏好锁失败"))
            }
        }
    }

    
}

