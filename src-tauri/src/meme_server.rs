use log::{debug, error, info};
use serde::{de, Deserialize, Serialize};
use std::time::Duration;
use tauri::Url;
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::{Client, Error, Method, Request, RequestBuilder, StatusCode};

use crate::meme_community::get_enabled_meme_libs;

/// 表情包项目的数据结构，与服务器返回的JSON对应
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemeItem {
    pub id: String,
    pub url: String,
    pub description: Option<String>,
}

/// 服务器返回的表情包搜索结果
#[derive(Debug, Serialize, Deserialize)]
// pub struct MemeSearchResponse {
//     pub code: i32,
//     pub message: String,
//     pub data: Vec<MemeItem>,
// }

/// 表情包服务器配置
pub struct MemeServerConfig {
    pub api_url: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
struct MemeSearchResponse {
    // code: u32,
    results: Vec<String>,
    // msg: String,
}

impl Default for MemeServerConfig {
    fn default() -> Self {
        Self {
            // 示例API URL，实际使用时需要替换为真实的API地址
            api_url: "https://api.example.com/memes".to_string(),
            timeout_seconds: 10,
        }
    }
}

/// 表情包服务客户端
pub struct MemeServerClient {
    client: reqwest::Client,
    config: MemeServerConfig,
}

impl MemeServerClient {
    /// 创建一个新的表情包服务客户端
    pub fn new(config: Option<MemeServerConfig>) -> Self {
        let config = config.unwrap_or_default();

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// 更新API URL
    pub fn update_api_url(&mut self, url: String) {
        self.config.api_url = url;
    }
    
    /// 搜索表情包
    pub async fn search_memes(&self, keyword: &str) -> Result<Vec<MemeItem>, Error> {
        // 先获取当前配置中的API URL
        let api_url = match crate::get_config_manager().get_active_api_url() {
            Ok(url) => url,
            Err(_) => self.config.api_url.clone(), // 如果获取失败，则使用默认值
        };

        info!("正在搜索表情包，关键词: {}", keyword);

        // 构建请求参数
        let payload = serde_json::json!({
            "query": keyword,
            "n_results": 10,  // 限制返回10个结果
            "resource_pack_uuids": match get_enabled_meme_libs() {
                Ok(libs) => libs,
                Err(e) => {
                    error!("获取已启用的表情包库失败: {}", e);
                    Vec::new() // 出错时使用空数组
                }
            }
        });

        debug!("发送请求到: {}", api_url);
        debug!("请求参数: {:?}", payload.to_string());

        // let query_string = serde_urlencoded::to_string(&payload).unwrap();

        // 构建完整URL（包括接口）
        let full_url = format!("{}/{}", api_url, "search");
        let url = Url::parse(&full_url).map_err(|e| {
            error!("请求失败: {}", e);
            e
        }).unwrap();
        debug!("完整请求URL: {}", url);

        let request_builder: RequestBuilder = self.client
            .request(Method::POST, url)
            .header("Content-Type", "application/json")
            .body(payload.to_string());

        let response = request_builder.send().await.map_err(|e| {
            error!("请求失败: {}", e);
            e
        })?;
        debug!("响应状态: {}", response.status());
        debug!("响应头: {:?}", response.headers());
        let json_data = response.text().await?;
        let meme_response: MemeSearchResponse = serde_json::from_str(&json_data)
            .map_err(|e| {
                error!("JSON解析失败: {}", e);
                e
            })
            .unwrap();
        debug!("响应体: {:?}", meme_response);

        // 将数据转换为 MemeItem 向量
        let meme_items: Vec<MemeItem> = meme_response
            .results
            .into_iter()
            .map(|url| MemeItem {
                id: uuid::Uuid::new_v4().to_string(), // 生成唯一ID
                url,
                description: None,
            })
            .collect();

        debug!("解析得到 {} 个表情包项目", meme_items.len());
        Ok(meme_items)

        // Ok(vec![])
        // Ok(json_data)
    }
}
