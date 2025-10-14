use crate::vpn_config::VpnConfigs;
use anyhow::Result;
use tokio::fs;

const CONFIG_PATH: &str = "vpn_configs.json";

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
