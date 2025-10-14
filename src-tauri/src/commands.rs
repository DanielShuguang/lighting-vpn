use crate::network_test::{
    batch_test_connections, batch_test_latencies, test_connection, test_http_connection,
    test_latency, BatchTestResult, TestResult,
};
use crate::proxy_manager::PROXY_MANAGER;
use crate::storage::{load_configs, save_configs};
use crate::subscription::{
    add_subscription, delete_subscription, load_subscriptions, refresh_subscription,
    update_subscription, Subscription, Subscriptions,
};
use crate::v2ray_config::{cleanup_config, generate_v2ray_config};
use crate::vpn_config::{export_config_url, parse_vpn_url, VpnConfig, VpnConfigs};
use crate::vpn_manager::{ConnectionInfo, VPN_MANAGER};
use anyhow::Result;

// ========== 配置管理命令 ==========

#[tauri::command]
pub async fn load_configs_command() -> Result<VpnConfigs, String> {
    load_configs()
        .await
        .map_err(|e| format!("Failed to load configs: {}", e))
}

#[tauri::command]
pub async fn save_configs_command(configs: VpnConfigs) -> Result<(), String> {
    save_configs(&configs)
        .await
        .map_err(|e| format!("Failed to save configs: {}", e))
}

#[tauri::command]
pub async fn parse_config_url_command(url: String) -> Result<VpnConfig, String> {
    parse_vpn_url(&url).map_err(|e| format!("Failed to parse URL: {}", e))
}

#[tauri::command]
pub async fn export_config_command(config: VpnConfig) -> Result<String, String> {
    export_config_url(&config).map_err(|e| format!("Failed to export config: {}", e))
}

// ========== 订阅管理命令 ==========

#[tauri::command]
pub async fn load_subscriptions_command() -> Result<Subscriptions, String> {
    load_subscriptions()
        .await
        .map_err(|e| format!("Failed to load subscriptions: {}", e))
}

#[tauri::command]
pub async fn add_subscription_command(
    name: String,
    url: String,
    use_proxy: bool,
    update_interval: u32,
) -> Result<Subscription, String> {
    add_subscription(name, url, use_proxy, update_interval)
        .await
        .map_err(|e| format!("Failed to add subscription: {}", e))
}

#[tauri::command]
pub async fn update_subscription_command(
    id: String,
    name: Option<String>,
    url: Option<String>,
    use_proxy: Option<bool>,
    update_interval: Option<u32>,
    enabled: Option<bool>,
) -> Result<Subscription, String> {
    update_subscription(id, name, url, use_proxy, update_interval, enabled)
        .await
        .map_err(|e| format!("Failed to update subscription: {}", e))
}

#[tauri::command]
pub async fn delete_subscription_command(id: String) -> Result<(), String> {
    delete_subscription(id)
        .await
        .map_err(|e| format!("Failed to delete subscription: {}", e))
}

#[tauri::command]
pub async fn refresh_subscription_command(
    id: String,
    proxy_url: Option<String>,
) -> Result<Vec<VpnConfig>, String> {
    refresh_subscription(id, proxy_url)
        .await
        .map_err(|e| format!("Failed to refresh subscription: {}", e))
}

// ========== 网络测试命令 ==========

#[tauri::command]
pub async fn test_connection_command(
    server: String,
    port: u16,
    timeout_secs: u64,
) -> Result<TestResult, String> {
    Ok(test_connection(&server, port, timeout_secs).await)
}

#[tauri::command]
pub async fn test_latency_command(
    server: String,
    port: u16,
    count: u32,
    timeout_secs: u64,
) -> Result<TestResult, String> {
    Ok(test_latency(&server, port, count, timeout_secs).await)
}

#[tauri::command]
pub async fn test_http_connection_command(
    url: String,
    timeout_secs: u64,
    use_proxy: bool,
    proxy_url: Option<String>,
) -> Result<TestResult, String> {
    Ok(test_http_connection(&url, timeout_secs, use_proxy, proxy_url.as_deref()).await)
}

#[tauri::command]
pub async fn batch_test_connections_command(
    configs: Vec<(String, String, String, u16)>,
    timeout_secs: u64,
) -> Result<Vec<BatchTestResult>, String> {
    Ok(batch_test_connections(configs, timeout_secs).await)
}

#[tauri::command]
pub async fn batch_test_latencies_command(
    configs: Vec<(String, String, String, u16)>,
    count: u32,
    timeout_secs: u64,
) -> Result<Vec<BatchTestResult>, String> {
    Ok(batch_test_latencies(configs, count, timeout_secs).await)
}

// ========== VPN 连接命令 ==========

#[tauri::command]
pub async fn connect_vpn_command(config: VpnConfig) -> Result<ConnectionInfo, String> {
    // 生成 V2Ray 配置文件
    let (config_path, http_port, socks_port) = generate_v2ray_config(&config)
        .await
        .map_err(|e| format!("生成配置失败: {}", e))?;

    let connection_info = ConnectionInfo {
        config_id: config.id.clone(),
        config_name: config.name.clone(),
        local_port: http_port,
        local_socks_port: socks_port,
        pid: None,
    };

    // 启动 V2Ray
    VPN_MANAGER
        .start_v2ray(&config_path, connection_info.clone())
        .await
        .map_err(|e| format!("启动 VPN 失败: {}", e))?;

    // 等待一段时间让进程启动
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 设置系统代理（在单独的作用域中处理，确保锁被释放）
    {
        let mut proxy_mgr = PROXY_MANAGER
            .lock()
            .map_err(|e| format!("获取代理管理器失败: {}", e))?;

        proxy_mgr
            .enable_proxy("127.0.0.1", http_port)
            .map_err(|e| format!("设置系统代理失败: {}", e))?;
    }

    // 返回更新后的连接信息（包含 PID）
    Ok(VPN_MANAGER
        .get_connection_info()
        .ok_or_else(|| "获取连接信息失败".to_string())?)
}

#[tauri::command]
pub async fn disconnect_vpn_command() -> Result<(), String> {
    // 获取配置 ID 用于清理
    let config_id = VPN_MANAGER.get_connection_info().map(|info| info.config_id);

    // 禁用系统代理（在单独的作用域中处理，确保锁被释放）
    {
        let mut proxy_mgr = PROXY_MANAGER
            .lock()
            .map_err(|e| format!("获取代理管理器失败: {}", e))?;

        proxy_mgr
            .disable_proxy()
            .map_err(|e| format!("禁用系统代理失败: {}", e))?;
    }

    // 停止 V2Ray 进程
    VPN_MANAGER
        .stop()
        .map_err(|e| format!("停止 VPN 失败: {}", e))?;

    // 清理配置文件
    if let Some(id) = config_id {
        cleanup_config(&id).await.ok();
    }

    Ok(())
}

#[tauri::command]
pub fn get_connection_status_command() -> Result<Option<ConnectionInfo>, String> {
    Ok(VPN_MANAGER.get_connection_info())
}

#[tauri::command]
pub fn is_connected_command() -> Result<bool, String> {
    Ok(VPN_MANAGER.is_connected())
}
