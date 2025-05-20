use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::{Client, Error, Method, Request, RequestBuilder, StatusCode};

// 定义manifest.json的数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct CommunityManifest {
    pub community_info: CommunityInfo,
    pub meme_libs: HashMap<String, MemeLib>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommunityInfo {
    pub resource_url: String,
    pub update_url: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemeLib {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub created_at: String,
    pub timestamp: u64,
    pub tags: Vec<String>,
    pub url: String,
    pub update_url: String,
    pub uuid: String,
}

// 添加表情包库启用状态配置结构
#[derive(Debug, Serialize, Deserialize)]
pub struct EnabledMemeLibs {
    pub enabled_libs: HashSet<String>,
}

impl Default for EnabledMemeLibs {
    fn default() -> Self {
        Self {
            enabled_libs: HashSet::new(),
        }
    }
}

// 定义下载manifest的函数
pub async fn download_community_manifest() -> Result<CommunityManifest, String> {
    const MANIFEST_URL: &str = "https://github.com/MemeMeow-Studio/Memes-Community/raw/main/community_manifest.json";
    info!("开始下载社区表情库清单");

    // 下载manifest文件
    let response = match reqwest::get(MANIFEST_URL).await {
        Ok(resp) => resp,
        Err(e) => {
            error!("下载社区表情库清单失败: {}", e);
            return Err(format!("下载失败: {}", e));
        }
    };

    if !response.status().is_success() {
        error!("下载社区表情库清单失败: 状态码 {}", response.status());
        return Err(format!("下载失败: 状态码 {}", response.status()));
    }

    // 解析JSON
    let manifest_text = match response.text().await {
        Ok(text) => text,
        Err(e) => {
            error!("读取社区表情库清单内容失败: {}", e);
            return Err(format!("读取内容失败: {}", e));
        }
    };

    let manifest: CommunityManifest = match serde_json::from_str(&manifest_text) {
        Ok(data) => data,
        Err(e) => {
            error!("解析社区表情库清单JSON失败: {}", e);
            return Err(format!("解析JSON失败: {}", e));
        }
    };

    // 将manifest保存到缓存目录
    if let Err(e) = save_manifest_to_cache(&manifest_text) {
        error!("保存社区表情库清单到缓存失败: {}", e);
        // 这里我们只记录错误，但不中断流程，因为我们已经有了内存中的数据
        debug!("将继续使用内存中的数据而不保存到缓存");
    }

    info!("社区表情库清单下载成功，包含 {} 个表情库", manifest.meme_libs.len());
    Ok(manifest)
}

// 保存manifest到缓存
fn save_manifest_to_cache(content: &str) -> Result<PathBuf, String> {
    // 使用 dirs 库获取缓存目录
    let cache_dir = match dirs::cache_dir() {
        Some(dir) => dir,
        None => {
            return Err("无法获取系统缓存目录".to_string());
        }
    };

    // 创建MemeMeow子目录
    let meme_cache_dir = cache_dir.join("MemeMeow");
    if !meme_cache_dir.exists() {
        if let Err(e) = fs::create_dir_all(&meme_cache_dir) {
            return Err(format!("创建缓存目录失败: {}", e));
        }
    }

    // 创建文件路径
    let file_path = meme_cache_dir.join("community_manifest.json");

    // 写入文件
    if let Err(e) = fs::write(&file_path, content) {
        return Err(format!("写入缓存文件失败: {}", e));
    }

    debug!("社区表情库清单已保存到: {:?}", file_path);
    Ok(file_path)
}

// 从缓存加载manifest
pub fn load_manifest_from_cache() -> Result<CommunityManifest, String> {
    // 使用 dirs 库获取缓存目录
    let cache_dir = match dirs::cache_dir() {
        Some(dir) => dir,
        None => {
            return Err("无法获取系统缓存目录".to_string());
        }
    };

    let file_path = cache_dir.join("MemeMeow").join("community_manifest.json");

    // 检查文件是否存在
    if !file_path.exists() {
        return Err("缓存文件不存在".to_string());
    }

    // 读取文件内容
    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) => {
            return Err(format!("读取缓存文件失败: {}", e));
        }
    };

    // 解析JSON
    match serde_json::from_str(&content) {
        Ok(manifest) => {
            debug!("从缓存加载社区表情库清单成功: {:?}", file_path);
            Ok(manifest)
        },
        Err(e) => Err(format!("解析缓存的JSON失败: {}", e)),
    }
}

// 获取启用状态配置文件路径
fn get_enabled_libs_path() -> Result<PathBuf, String> {
    let config_dir = match dirs::config_dir() {
        Some(dir) => dir,
        None => {
            return Err("无法获取系统配置目录".to_string());
        }
    };

    // 创建MemeMeow子目录
    let meme_config_dir = config_dir.join("MemeMeow");
    if !meme_config_dir.exists() {
        if let Err(e) = fs::create_dir_all(&meme_config_dir) {
            return Err(format!("创建配置目录失败: {}", e));
        }
    }

    // 返回文件路径
    Ok(meme_config_dir.join("enabled_meme_libs.json"))
}

// 加载已启用的表情库
pub fn load_enabled_meme_libs() -> Result<EnabledMemeLibs, String> {
    let file_path = get_enabled_libs_path()?;

    // 如果文件不存在，返回默认空列表
    if !file_path.exists() {
        info!("启用的表情库配置文件不存在，将创建新文件");
        let default_config = EnabledMemeLibs::default();
        save_enabled_meme_libs(&default_config)?;
        return Ok(default_config);
    }

    // 读取文件内容
    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) => {
            error!("读取启用的表情库配置失败: {}", e);
            return Err(format!("读取配置失败: {}", e));
        }
    };

    // 解析JSON
    match serde_json::from_str(&content) {
        Ok(libs) => {
            debug!("成功加载启用的表情库配置");
            Ok(libs)
        },
        Err(e) => {
            error!("解析启用的表情库配置失败: {}", e);
            // 如果解析失败，返回默认值
            Ok(EnabledMemeLibs::default())
        }
    }
}

// 表情包社区相关命令
#[tauri::command]
pub async fn fetch_community_manifest() -> Result<CommunityManifest, String> {
    info!("接收到获取社区表情库清单请求");
    
    // 先尝试从缓存加载
    match load_manifest_from_cache() {
        Ok(manifest) => {
            info!("从缓存加载社区表情库清单成功");
            Ok(manifest)
        }
        Err(e) => {
            debug!("从缓存加载失败: {}，将从网络下载", e);
            // 缓存加载失败，从网络下载
            download_community_manifest().await
        }
    }
}

// 保存已启用的表情库
pub fn save_enabled_meme_libs(enabled_libs: &EnabledMemeLibs) -> Result<(), String> {
    let file_path = get_enabled_libs_path()?;

    // 序列化为JSON
    let json_content = match serde_json::to_string_pretty(enabled_libs) {
        Ok(content) => content,
        Err(e) => {
            error!("序列化启用的表情库配置失败: {}", e);
            return Err(format!("序列化配置失败: {}", e));
        }
    };

    // 写入文件
    if let Err(e) = fs::write(&file_path, json_content) {
        error!("保存启用的表情库配置失败: {}", e);
        return Err(format!("保存配置失败: {}", e));
    }

    debug!("成功保存启用的表情库配置到: {:?}", file_path);
    Ok(())
}

// 启用表情库
#[tauri::command]
pub fn enable_meme_lib(uuid: &str) -> Result<(), String> {
    let mut enabled_libs = load_enabled_meme_libs()?;
    
    // 添加到集合中
    enabled_libs.enabled_libs.insert(uuid.to_string());
    
    // 保存更改
    save_enabled_meme_libs(&enabled_libs)?;
    info!("表情库已启用: {}", uuid);
    
    Ok(())
}

// 禁用表情库
#[tauri::command]
pub fn disable_meme_lib(uuid: &str) -> Result<(), String> {
    let mut enabled_libs = load_enabled_meme_libs()?;
    
    // 从集合中移除
    enabled_libs.enabled_libs.remove(uuid);
    
    // 保存更改
    save_enabled_meme_libs(&enabled_libs)?;
    info!("表情库已禁用: {}", uuid);
    
    Ok(())
}

// 获取所有已启用的表情库UUID列表
#[tauri::command]
pub fn get_enabled_meme_libs() -> Result<Vec<String>, String> {
    let enabled_libs = load_enabled_meme_libs()?;
    Ok(enabled_libs.enabled_libs.into_iter().collect())
}

#[tauri::command]
pub async fn refresh_community_manifest() -> Result<CommunityManifest, String> {
    info!("接收到刷新社区表情库清单请求");
    // 强制从网络刷新
    download_community_manifest().await
}