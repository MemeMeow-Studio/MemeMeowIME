use serde::{de, Deserialize, Serialize};
use std::fs::{self, File};
use std::path::PathBuf;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use log::{debug, error, info};
use tauri_plugin_global_shortcut::{Code, Modifiers};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShortcutConfig {
    pub modifiers: Vec<String>,
    pub key: String,
    pub action: String,
}

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
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ShortcutConfigs {
    #[serde(default = "default_toggle_app_shortcut")]
    pub toggle_app: ShortcutConfig,
    // 可以添加更多快捷键配置
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

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            copy_to_clipboard: true,
            shortcuts: ShortcutConfigs::default(),
        }
    }
}

impl ShortcutConfig {
    // 将配置转换为Tauri快捷键代码
    pub fn to_tauri_shortcut(&self) -> (Modifiers, Code) {
        let mut modifiers = Modifiers::empty();
        for modifier in &self.modifiers {
            match modifier.to_lowercase().as_str() {
                "ctrl" => modifiers.insert(Modifiers::CONTROL),
                "alt" => modifiers.insert(Modifiers::ALT),
                "shift" => modifiers.insert(Modifiers::SHIFT),
                "meta" | "super" | "command" => modifiers.insert(Modifiers::META),
                _ => continue,
            }
        }
        
        // 将字符串键转换为Tauri Code
        let code = match self.key.to_lowercase().as_str() {
            "a" => Code::KeyA,
            "b" => Code::KeyB,
            "c" => Code::KeyC,
            "d" => Code::KeyD,
            "e" => Code::KeyE,
            "f" => Code::KeyF,
            "g" => Code::KeyG,
            "h" => Code::KeyH,
            "i" => Code::KeyI,
            "j" => Code::KeyJ,
            "k" => Code::KeyK,
            "l" => Code::KeyL,
            "m" => Code::KeyM,
            "n" => Code::KeyN,
            "o" => Code::KeyO,
            "p" => Code::KeyP,
            "q" => Code::KeyQ,
            "r" => Code::KeyR,
            "s" => Code::KeyS,
            "t" => Code::KeyT,
            "u" => Code::KeyU,
            "v" => Code::KeyV,
            "w" => Code::KeyW,
            "x" => Code::KeyX,
            "y" => Code::KeyY,
            "z" => Code::KeyZ,
            // 数字键
            "0" => Code::Digit0,
            "1" => Code::Digit1,
            "2" => Code::Digit2,
            "3" => Code::Digit3,
            "4" => Code::Digit4,
            "5" => Code::Digit5,
            "6" => Code::Digit6,
            "7" => Code::Digit7,
            "8" => Code::Digit8,
            "9" => Code::Digit9,
            // 功能键
            "f1" => Code::F1,
            "f2" => Code::F2,
            "f3" => Code::F3,
            "f4" => Code::F4,
            "f5" => Code::F5,
            "f6" => Code::F6,
            "f7" => Code::F7,
            "f8" => Code::F8,
            "f9" => Code::F9,
            "f10" => Code::F10,
            "f11" => Code::F11,
            "f12" => Code::F12,
            // 默认键
            _ => Code::KeyV, // 默认使用V键
        };
        
        (modifiers, code)
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
            },
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
            },
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
            },
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
            },
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
}
