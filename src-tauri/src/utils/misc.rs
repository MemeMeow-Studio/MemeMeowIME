use serde::{de, Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiUrl {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiServerUrlsConfig {
    #[serde(default = "default_api_server_urls")]
    pub urls: Vec<ApiUrl>,
}

fn default_api_server_urls() -> Vec<ApiUrl> {
    vec![
        ApiUrl {
            name: "默认API".to_string(),
            url: "https://mememeow.morami.icu".to_string(),
        },
    ]
}