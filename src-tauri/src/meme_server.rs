use log::{debug, error, info};
use serde::{de, Deserialize, Serialize};
use std::time::Duration;
use tauri::Url;
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::{Client, Error, Method, Request, RequestBuilder, StatusCode};

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
    code: u32,
    data: Vec<String>,
    msg: String,
}

impl Default for MemeServerConfig {
    fn default() -> Self {
        Self {
            // 示例API URL，实际使用时需要替换为真实的API地址
            api_url: "https://api.example.com/memes/search".to_string(),
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

    /// 搜索表情包
    pub async fn search_memes(&self, keyword: &str) -> Result<Vec<MemeItem>, Error> {
        // -> Result<Vec<MemeItem>, Error>
        info!("正在搜索表情包，关键词: {}", keyword);

        // 构建请求参数
        let payload = serde_json::json!({
            "q": keyword,
            "n": 10  // 限制返回10个结果
        });

        debug!("发送请求到: {}", self.config.api_url);
        debug!("请求参数: {:?}", payload);

        let query_string = serde_urlencoded::to_string(&payload).unwrap();

        // 构建完整URL（包括查询参数）
        let full_url = format!("https://api.zvv.quest/search?{}", query_string);
        let url = Url::parse(&full_url).unwrap();

        debug!("完整请求URL: {}", url);

        // // 发送POST请求到表情包服务器
        // let payload_str = payload.to_string();

        // // 创建请求体
        // let body = tauri_plugin_http::reqwest::Body::from(payload_str);

        // // 构建 URL
        // let url = Url::parse("https://api.zvv.quest/search").unwrap();// .map_err(|e| e.to_string())?;

        let request_builder: RequestBuilder = Client::new()
            .request(Method::GET, url)
            .header("Content-Type", "application/json");
        // .json(&payload);

        let response = request_builder.send().await.map_err(|e| {
            error!("请求失败: {}", e);
            e
        })?;
        debug!("响应状态: {}", response.status());
        debug!("响应头: {:?}", response.headers());
        let json_data = response.text().await?; // response.json::<MemeSearchResponse>().await?.data;
        let meme_response: MemeSearchResponse = serde_json::from_str(&json_data)
            .map_err(|e| {
                error!("JSON解析失败: {}", e);
                e
            })
            .unwrap();
        debug!("响应体: {:?}", meme_response);

        // 将数据转换为 MemeItem 向量
        let meme_items: Vec<MemeItem> = meme_response
            .data
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
