use tauri_plugin_global_shortcut::{Code, Modifiers};
use serde::{de, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShortcutConfig {
    pub modifiers: Vec<String>,
    pub key: String,
    pub action: String,
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
