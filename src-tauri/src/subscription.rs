use crate::vpn_config::{parse_vpn_url, VpnConfig};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use tokio::fs;

const SUBSCRIPTION_PATH: &str = "subscriptions.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub use_proxy: bool,
    pub update_interval: u32, // 单位：小时
    pub last_update: Option<chrono::DateTime<chrono::Utc>>,
    pub config_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscriptions {
    pub subscriptions: Vec<Subscription>,
}

impl Subscriptions {
    pub fn new() -> Self {
        Self {
            subscriptions: Vec::new(),
        }
    }
}

// 从订阅地址获取配置列表
pub async fn fetch_subscription_configs(
    url: &str,
    use_proxy: bool,
    proxy_url: Option<&str>,
) -> Result<Vec<VpnConfig>> {
    let mut client_builder = reqwest::Client::builder().timeout(std::time::Duration::from_secs(30));

    // 如果使用代理，配置代理设置
    if use_proxy {
        if let Some(proxy) = proxy_url {
            let proxy = reqwest::Proxy::all(proxy)?;
            client_builder = client_builder.proxy(proxy);
        }
    }

    let client = client_builder.build()?;
    let response = client.get(url).send().await?;
    let content = response.text().await?;

    parse_subscription_content(&content)
}

// 解析订阅内容
fn parse_subscription_content(content: &str) -> Result<Vec<VpnConfig>> {
    let mut configs = Vec::new();

    // 尝试 Base64 解码
    let decoded_content = if let Ok(decoded) = general_purpose::STANDARD.decode(content.trim()) {
        String::from_utf8(decoded).unwrap_or_else(|_| content.to_string())
    } else {
        content.to_string()
    };

    // 按行分割，解析每一行
    for line in decoded_content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // 尝试解析为 VPN 配置
        if let Ok(config) = parse_vpn_url(line) {
            configs.push(config);
        }
    }

    Ok(configs)
}

// 加载订阅列表
pub async fn load_subscriptions() -> Result<Subscriptions> {
    if fs::metadata(SUBSCRIPTION_PATH).await.is_err() {
        return Ok(Subscriptions::new());
    }

    let content = fs::read_to_string(SUBSCRIPTION_PATH).await?;
    let subscriptions: Subscriptions = serde_json::from_str(&content)?;

    Ok(subscriptions)
}

// 保存订阅列表
pub async fn save_subscriptions(subscriptions: &Subscriptions) -> Result<()> {
    let content = serde_json::to_string_pretty(subscriptions)?;
    fs::write(SUBSCRIPTION_PATH, content).await?;
    Ok(())
}

// 添加订阅
pub async fn add_subscription(
    name: String,
    url: String,
    use_proxy: bool,
    update_interval: u32,
) -> Result<Subscription> {
    let subscription = Subscription {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        url,
        enabled: true,
        use_proxy,
        update_interval,
        last_update: None,
        config_count: 0,
        created_at: chrono::Utc::now(),
    };

    let mut subscriptions = load_subscriptions().await?;
    subscriptions.subscriptions.push(subscription.clone());
    save_subscriptions(&subscriptions).await?;

    Ok(subscription)
}

// 更新订阅
pub async fn update_subscription(
    id: String,
    name: Option<String>,
    url: Option<String>,
    use_proxy: Option<bool>,
    update_interval: Option<u32>,
    enabled: Option<bool>,
) -> Result<Subscription> {
    let mut subscriptions = load_subscriptions().await?;

    let subscription = subscriptions
        .subscriptions
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| anyhow::anyhow!("Subscription not found"))?;

    if let Some(name) = name {
        subscription.name = name;
    }
    if let Some(url) = url {
        subscription.url = url;
    }
    if let Some(use_proxy) = use_proxy {
        subscription.use_proxy = use_proxy;
    }
    if let Some(update_interval) = update_interval {
        subscription.update_interval = update_interval;
    }
    if let Some(enabled) = enabled {
        subscription.enabled = enabled;
    }

    let updated_subscription = subscription.clone();
    save_subscriptions(&subscriptions).await?;

    Ok(updated_subscription)
}

// 删除订阅
pub async fn delete_subscription(id: String) -> Result<()> {
    let mut subscriptions = load_subscriptions().await?;
    subscriptions.subscriptions.retain(|s| s.id != id);
    save_subscriptions(&subscriptions).await?;
    Ok(())
}

// 更新订阅的配置列表
pub async fn refresh_subscription(id: String, proxy_url: Option<String>) -> Result<Vec<VpnConfig>> {
    let mut subscriptions = load_subscriptions().await?;

    let subscription = subscriptions
        .subscriptions
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or_else(|| anyhow::anyhow!("Subscription not found"))?;

    let configs = fetch_subscription_configs(
        &subscription.url,
        subscription.use_proxy,
        proxy_url.as_deref(),
    )
    .await?;

    subscription.last_update = Some(chrono::Utc::now());
    subscription.config_count = configs.len();

    save_subscriptions(&subscriptions).await?;

    Ok(configs)
}
