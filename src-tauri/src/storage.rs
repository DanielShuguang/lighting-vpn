use crate::vpn_config::VpnConfigs;
use anyhow::Result;
use std::fs as std_fs;
use tokio::fs;

const CONFIG_PATH: &str = "vpn_configs.json";
const PROXY_MODE_PATH: &str = "proxy_mode.txt";

pub async fn load_configs() -> Result<VpnConfigs> {
    if fs::metadata(CONFIG_PATH).await.is_err() {
        return Ok(VpnConfigs::new());
    }

    let content = fs::read_to_string(CONFIG_PATH).await?;
    let configs: VpnConfigs = serde_json::from_str(&content)?;

    Ok(configs)
}

pub async fn save_configs(configs: &VpnConfigs) -> Result<()> {
    let content = serde_json::to_string_pretty(configs)?;
    fs::write(CONFIG_PATH, content).await?;
    Ok(())
}

/// 保存代理模式
pub fn save_proxy_mode(mode: &str) -> Result<()> {
    std_fs::write(PROXY_MODE_PATH, mode)?;
    Ok(())
}

/// 加载代理模式
pub fn load_proxy_mode() -> Result<String> {
    if !std::path::Path::new(PROXY_MODE_PATH).exists() {
        return Ok("pac".to_string()); // 默认 PAC 模式
    }

    let mode = std_fs::read_to_string(PROXY_MODE_PATH)?;
    Ok(mode.trim().to_string())
}
