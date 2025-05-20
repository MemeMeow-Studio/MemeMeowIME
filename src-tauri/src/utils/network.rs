use log::{debug, error, info, warn};
use std::time::Duration;
use tauri_plugin_http::reqwest::{self, Client, StatusCode};

/// 尝试从多个URL下载文件，使用渐进式超时策略
/// 
/// - 初始超时设置为1秒
/// - 如果所有URL都尝试失败，增加超时时间并重试
/// - 最大超时时间为10秒
/// 
/// # 参数
/// 
/// * `urls` - 要尝试下载的URL，可以是单个String或Vec<String>
/// 
/// # 返回值
/// 
/// * `Ok(String)` - 下载成功的文件内容
/// * `Err(String)` - 下载失败的错误信息
pub async fn download_with_fallback_urls<I, S>(urls: I) -> Result<String, String> 
where 
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    // 收集URL到Vec中以便多次使用
    let urls: Vec<String> = urls
        .into_iter()
        .map(|s| s.as_ref().to_string())
        .collect();
    
    if urls.is_empty() {
        return Err("URL列表为空".to_string());
    }
    

    let mut timeout = 3;
    let max_timeout = 10;
    
    while timeout <= max_timeout {
        debug!("使用{}秒超时尝试下载", timeout);
        
        // 创建带有自定义超时的客户端
        let client = match Client::builder()
            .timeout(Duration::from_secs(timeout))
            .build() {
                Ok(client) => client,
                Err(e) => return Err(format!("创建HTTP客户端失败: {}", e)),
            };
            
        // 尝试每个URL
        for (i, url) in urls.iter().enumerate() {
            debug!("尝试URL {}/{}: {}", i+1, urls.len(), url);
            
            match download_single_url(&client, url).await {
                Ok(content) => {
                    info!("成功从URL下载内容: {}", url);
                    return Ok(content);
                },
                Err(e) => {
                    warn!("从URL下载失败: {} - 错误: {}", url, e);
                    // 继续尝试下一个URL
                }
            }
        }
        
        // 如果所有URL都失败了，增加超时时间
        timeout *= 2;
        if timeout <= max_timeout {
            warn!("所有URL下载失败，增加超时时间至{}秒后重试", timeout);
        }
    }
    
    error!("所有URL在所有超时设置下均下载失败");
    Err("无法从任何提供的URL下载内容".to_string())
}

/// 从单个URL下载内容
async fn download_single_url(client: &Client, url: &str) -> Result<String, String> {
    // 发起请求
    let response = match client.get(url).send().await {
        Ok(resp) => resp,
        Err(e) => {
            return Err(format!("请求失败: {}", e));
        }
    };
    
    // 检查状态码
    if !response.status().is_success() {
        return Err(format!("状态码错误: {}", response.status()));
    }
    
    // 获取响应内容
    match response.text().await {
        Ok(text) => Ok(text),
        Err(e) => Err(format!("读取响应内容失败: {}", e))
    }
}
