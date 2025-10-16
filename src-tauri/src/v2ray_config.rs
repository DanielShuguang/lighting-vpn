use crate::vpn_config::{VpnConfig, VpnProtocol};
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use tokio::fs;

const LOCAL_HTTP_PORT: u16 = 10809;
const LOCAL_SOCKS_PORT: u16 = 10808;

// 生成 V2Ray 配置文件
pub async fn generate_v2ray_config(config: &VpnConfig) -> Result<(String, u16, u16)> {
    // 使用系统临时目录以避免触发文件监视器
    let temp_dir = std::env::temp_dir();
    let config_dir = temp_dir.join("vpn_configs");

    // 确保配置目录存在
    fs::create_dir_all(&config_dir).await.ok();

    let v2ray_config = match config.protocol {
        VpnProtocol::Vmess => generate_vmess_config(config)?,
        VpnProtocol::Shadowsocks => generate_shadowsocks_config(config)?,
        VpnProtocol::Trojan => generate_trojan_config(config)?,
        VpnProtocol::V2Ray => generate_vless_config(config)?,
        _ => return Err(anyhow!("不支持的协议: {:?}", config.protocol)),
    };

    // 保存配置文件到临时目录
    let config_path = config_dir.join(format!("config_{}.json", config.id));
    let config_json = serde_json::to_string_pretty(&v2ray_config)?;
    fs::write(&config_path, config_json).await?;

    Ok((
        config_path.to_string_lossy().to_string(),
        LOCAL_HTTP_PORT,
        LOCAL_SOCKS_PORT,
    ))
}

// 生成 VMess 配置
fn generate_vmess_config(config: &VpnConfig) -> Result<Value> {
    let uuid = config
        .password
        .as_ref()
        .ok_or_else(|| anyhow!("VMess 协议缺少 UUID。请确保配置的密码字段包含有效的 UUID"))?;

    Ok(json!({
        "log": {
            "loglevel": "warning"
        },
        "inbounds": [
            {
                "port": LOCAL_HTTP_PORT,
                "protocol": "http",
                "settings": {
                    "timeout": 300
                }
            },
            {
                "port": LOCAL_SOCKS_PORT,
                "protocol": "socks",
                "settings": {
                    "auth": "noauth",
                    "udp": true
                }
            }
        ],
        "outbounds": [
            {
                "protocol": "vmess",
                "settings": {
                    "vnext": [
                        {
                            "address": config.server,
                            "port": config.port,
                            "users": [
                                {
                                    "id": uuid,
                                    "alterId": 0,
                                    "security": "auto"
                                }
                            ]
                        }
                    ]
                },
                "streamSettings": {
                    "network": "tcp"
                }
            }
        ]
    }))
}

// 生成 Shadowsocks 配置
fn generate_shadowsocks_config(config: &VpnConfig) -> Result<Value> {
    let password = config
        .password
        .as_ref()
        .ok_or_else(|| anyhow!("Shadowsocks 协议缺少密码"))?;
    let method = config
        .method
        .as_ref()
        .ok_or_else(|| anyhow!("Shadowsocks 协议缺少加密方法"))?;

    Ok(json!({
        "log": {
            "loglevel": "warning"
        },
        "inbounds": [
            {
                "port": LOCAL_HTTP_PORT,
                "protocol": "http",
                "settings": {
                    "timeout": 300
                }
            },
            {
                "port": LOCAL_SOCKS_PORT,
                "protocol": "socks",
                "settings": {
                    "auth": "noauth",
                    "udp": true
                }
            }
        ],
        "outbounds": [
            {
                "protocol": "shadowsocks",
                "settings": {
                    "servers": [
                        {
                            "address": config.server,
                            "port": config.port,
                            "method": method,
                            "password": password
                        }
                    ]
                }
            }
        ]
    }))
}

// 生成 Trojan 配置
fn generate_trojan_config(config: &VpnConfig) -> Result<Value> {
    let password = config
        .password
        .as_ref()
        .ok_or_else(|| anyhow!("Trojan 协议缺少密码"))?;

    Ok(json!({
        "log": {
            "loglevel": "warning"
        },
        "inbounds": [
            {
                "port": LOCAL_HTTP_PORT,
                "protocol": "http",
                "settings": {
                    "timeout": 300
                }
            },
            {
                "port": LOCAL_SOCKS_PORT,
                "protocol": "socks",
                "settings": {
                    "auth": "noauth",
                    "udp": true
                }
            }
        ],
        "outbounds": [
            {
                "protocol": "trojan",
                "settings": {
                    "servers": [
                        {
                            "address": config.server,
                            "port": config.port,
                            "password": password
                        }
                    ]
                },
                "streamSettings": {
                    "network": "tcp",
                    "security": "tls",
                    "tlsSettings": {
                        "allowInsecure": false
                    }
                }
            }
        ]
    }))
}

// 生成 VLESS 配置
fn generate_vless_config(config: &VpnConfig) -> Result<Value> {
    let uuid = config
        .password
        .as_ref()
        .ok_or_else(|| anyhow!("VLess 协议缺少 UUID。请确保配置的密码字段包含有效的 UUID"))?;

    Ok(json!({
        "log": {
            "loglevel": "warning"
        },
        "inbounds": [
            {
                "port": LOCAL_HTTP_PORT,
                "protocol": "http",
                "settings": {
                    "timeout": 300
                }
            },
            {
                "port": LOCAL_SOCKS_PORT,
                "protocol": "socks",
                "settings": {
                    "auth": "noauth",
                    "udp": true
                }
            }
        ],
        "outbounds": [
            {
                "protocol": "vless",
                "settings": {
                    "vnext": [
                        {
                            "address": config.server,
                            "port": config.port,
                            "users": [
                                {
                                    "id": uuid,
                                    "encryption": "none"
                                }
                            ]
                        }
                    ]
                },
                "streamSettings": {
                    "network": "tcp"
                }
            }
        ]
    }))
}

// 清理配置文件
pub async fn cleanup_config(config_id: &str) -> Result<()> {
    let temp_dir = std::env::temp_dir();
    let config_dir = temp_dir.join("vpn_configs");
    let config_path = config_dir.join(format!("config_{}.json", config_id));

    if config_path.exists() {
        fs::remove_file(&config_path).await?;
    }
    Ok(())
}
