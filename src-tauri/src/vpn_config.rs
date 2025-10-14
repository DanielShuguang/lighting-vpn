use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnConfig {
    pub id: String,
    pub name: String,
    pub protocol: VpnProtocol,
    pub server: String,
    pub port: u16,
    pub password: Option<String>,
    pub method: Option<String>,
    pub remarks: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VpnProtocol {
    Shadowsocks,
    ShadowsocksR,
    V2Ray,
    Trojan,
    Vmess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnConfigs {
    pub configs: Vec<VpnConfig>,
}

impl VpnConfigs {
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
        }
    }
}

// 解析Shadowsocks URL
fn parse_shadowsocks_url(url: &str) -> Result<VpnConfig> {
    let parsed_url = Url::parse(url)?;
    let host = parsed_url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid host"))?;
    let port = parsed_url.port().unwrap_or(8388);

    // 解析认证信息
    let auth = parsed_url.username();
    let decoded = general_purpose::STANDARD.decode(auth)?;
    let auth_str = String::from_utf8(decoded)?;

    let parts: Vec<&str> = auth_str.split(':').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid auth format"));
    }

    let method = parts[0].to_string();
    let password = parts[1].to_string();

    Ok(VpnConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: parsed_url.fragment().unwrap_or("Shadowsocks").to_string(),
        protocol: VpnProtocol::Shadowsocks,
        server: host.to_string(),
        port,
        password: Some(password),
        method: Some(method),
        remarks: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

// 解析ShadowsocksR URL
fn parse_shadowsocksr_url(url: &str) -> Result<VpnConfig> {
    // SSR URL格式: ssr://base64(server:port:protocol:method:obfs:base64(password)/base64(remarks))
    let url = url
        .strip_prefix("ssr://")
        .ok_or_else(|| anyhow::anyhow!("Invalid SSR URL"))?;
    let decoded = general_purpose::STANDARD.decode(url)?;
    let config_str = String::from_utf8(decoded)?;

    // 解析SSR配置
    let parts: Vec<&str> = config_str.split('/').collect();
    if parts.len() < 2 {
        return Err(anyhow::anyhow!("Invalid SSR format"));
    }

    let main_part = parts[0];
    let remarks_part = if parts.len() > 1 { parts[1] } else { "" };

    let main_parts: Vec<&str> = main_part.split(':').collect();
    if main_parts.len() < 6 {
        return Err(anyhow::anyhow!("Invalid SSR main part"));
    }

    let server = main_parts[0];
    let port: u16 = main_parts[1]
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid port"))?;
    let protocol = main_parts[2];
    let method = main_parts[3];
    let obfs = main_parts[4];
    let password_b64 = main_parts[5];

    // 解码密码
    let password = general_purpose::STANDARD.decode(password_b64)?;
    let password_str = String::from_utf8(password)?;

    // 解码备注
    let remarks = if !remarks_part.is_empty() {
        let decoded_remarks = general_purpose::STANDARD.decode(remarks_part)?;
        Some(String::from_utf8(decoded_remarks)?)
    } else {
        None
    };

    Ok(VpnConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: remarks
            .clone()
            .unwrap_or_else(|| "ShadowsocksR".to_string()),
        protocol: VpnProtocol::ShadowsocksR,
        server: server.to_string(),
        port,
        password: Some(password_str),
        method: Some(format!("{}:{}:{}", method, protocol, obfs)),
        remarks,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

// 解析V2Ray URL
fn parse_v2ray_url(url: &str) -> Result<VpnConfig> {
    let parsed_url = Url::parse(url)?;
    let host = parsed_url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid host"))?;
    let port = parsed_url.port().unwrap_or(443);

    // 解析查询参数
    let query_pairs: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();

    let remarks = query_pairs.get("remarks").cloned();
    let password = query_pairs.get("password").cloned();

    Ok(VpnConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: remarks.clone().unwrap_or_else(|| "V2Ray".to_string()),
        protocol: VpnProtocol::V2Ray,
        server: host.to_string(),
        port,
        password,
        method: None,
        remarks,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

// 解析VMess URL
fn parse_vmess_url(url: &str) -> Result<VpnConfig> {
    // VMess URL格式: vmess://base64_encoded_config
    let url = url
        .strip_prefix("vmess://")
        .ok_or_else(|| anyhow::anyhow!("Invalid VMess URL"))?;
    let decoded = general_purpose::STANDARD.decode(url)?;
    let config_str = String::from_utf8(decoded)?;

    let config: serde_json::Value = serde_json::from_str(&config_str)?;

    Ok(VpnConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: config["ps"].as_str().unwrap_or("VMess").to_string(),
        protocol: VpnProtocol::Vmess,
        server: config["add"].as_str().unwrap_or("").to_string(),
        port: config["port"].as_u64().unwrap_or(443) as u16,
        password: config["id"].as_str().map(|s| s.to_string()),
        method: None,
        remarks: config["ps"].as_str().map(|s| s.to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

// 解析Trojan URL
fn parse_trojan_url(url: &str) -> Result<VpnConfig> {
    let parsed_url = Url::parse(url)?;
    let host = parsed_url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid host"))?;
    let port = parsed_url.port().unwrap_or(443);
    let password = parsed_url.username().to_string();

    let query_pairs: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();
    let remarks = query_pairs.get("remarks").cloned();

    Ok(VpnConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: remarks.clone().unwrap_or_else(|| "Trojan".to_string()),
        protocol: VpnProtocol::Trojan,
        server: host.to_string(),
        port,
        password: Some(password),
        method: None,
        remarks,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

// 通用URL解析函数
pub fn parse_vpn_url(url: &str) -> Result<VpnConfig> {
    if url.starts_with("ss://") {
        parse_shadowsocks_url(url)
    } else if url.starts_with("ssr://") {
        parse_shadowsocksr_url(url)
    } else if url.starts_with("vmess://") {
        parse_vmess_url(url)
    } else if url.starts_with("trojan://") {
        parse_trojan_url(url)
    } else if url.starts_with("vless://") {
        // VLess解析逻辑
        parse_v2ray_url(url)
    } else {
        Err(anyhow::anyhow!("Unsupported protocol"))
    }
}

// 导出配置为URL
pub fn export_config_url(config: &VpnConfig) -> Result<String> {
    match config.protocol {
        VpnProtocol::Shadowsocks => {
            let auth = format!(
                "{}:{}",
                config.method.as_ref().unwrap_or(&"aes-256-gcm".to_string()),
                config.password.as_ref().unwrap_or(&"".to_string())
            );
            let encoded_auth = general_purpose::STANDARD.encode(auth);
            Ok(format!(
                "ss://{}@{}:{}#{}",
                encoded_auth,
                config.server,
                config.port,
                urlencoding::encode(&config.name)
            ))
        }
        VpnProtocol::ShadowsocksR => {
            // 解析method字段获取协议、加密方法、混淆
            let default_method = "aes-256-cfb:origin:plain".to_string();
            let method_str = config.method.as_ref().unwrap_or(&default_method);
            let method_parts: Vec<&str> = method_str.split(':').collect();

            let (method, protocol, obfs) = if method_parts.len() == 3 {
                (method_parts[0], method_parts[1], method_parts[2])
            } else {
                ("aes-256-cfb", "origin", "plain")
            };

            // 编码密码和备注
            let password_b64 = general_purpose::STANDARD
                .encode(config.password.as_ref().unwrap_or(&"".to_string()));
            let remarks_b64 = general_purpose::STANDARD.encode(&config.name);

            // 构建SSR配置字符串
            let ssr_config = format!(
                "{}:{}:{}:{}:{}:{}/{}",
                config.server, config.port, protocol, method, obfs, password_b64, remarks_b64
            );

            let encoded_config = general_purpose::STANDARD.encode(ssr_config);
            Ok(format!("ssr://{}", encoded_config))
        }
        VpnProtocol::Vmess => {
            let vmess_config = serde_json::json!({
                "v": "2",
                "ps": config.name,
                "add": config.server,
                "port": config.port,
                "id": config.password.clone().unwrap_or_default(),
                "aid": "0",
                "scy": "auto",
                "net": "tcp",
                "type": "none",
                "host": "",
                "path": "",
                "tls": "none"
            });
            let config_str = serde_json::to_string(&vmess_config)
                .map_err(|e| anyhow::anyhow!("Failed to serialize VMess config: {}", e))?;
            let encoded = general_purpose::STANDARD.encode(config_str);
            Ok(format!("vmess://{}", encoded))
        }
        _ => Err(anyhow::anyhow!("Unsupported protocol for export")),
    }
}
