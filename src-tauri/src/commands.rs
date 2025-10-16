use crate::core_manager::{
    check_core_installed, download_core, remove_core, CoreInfo, DownloadProgress,
};
use crate::network_test::{
    batch_test_connections, batch_test_latencies, test_connection, test_http_connection,
    test_latency, BatchTestResult, TestResult,
};
use crate::proxy_manager::{ProxyMode, PROXY_MANAGER};
use crate::storage::{load_configs, load_proxy_mode, save_configs, save_proxy_mode};
use crate::subscription::{
    add_subscription, delete_subscription, load_subscriptions, refresh_subscription,
    update_subscription, Subscription, Subscriptions,
};
use crate::v2ray_config::{cleanup_config, generate_v2ray_config};
use crate::vpn_config::{export_config_url, parse_vpn_url, VpnConfig, VpnConfigs};
use crate::vpn_manager::{ConnectionInfo, VPN_MANAGER};
use anyhow::Result;
use tauri::Emitter;

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
    println!("[VPN] 开始连接: {}", config.name);

    // 生成 V2Ray 配置文件
    let (config_path, http_port, socks_port) =
        generate_v2ray_config(&config).await.map_err(|e| {
            eprintln!("[VPN] 生成配置失败: {}", e);
            format!("生成配置失败: {}", e)
        })?;

    println!("[VPN] 配置文件已生成: {}", config_path);
    println!("[VPN] HTTP端口: {}, SOCKS端口: {}", http_port, socks_port);

    let connection_info = ConnectionInfo {
        config_id: config.id.clone(),
        config_name: config.name.clone(),
        local_port: http_port,
        local_socks_port: socks_port,
        pid: None,
    };

    // 启动 V2Ray
    println!("[VPN] 正在启动 V2Ray 进程...");
    VPN_MANAGER
        .start_v2ray(&config_path, connection_info.clone())
        .await
        .map_err(|e| {
            eprintln!("[VPN] 启动进程失败: {}", e);
            format!("启动 VPN 失败: {}", e)
        })?;

    println!("[VPN] V2Ray 进程已启动");

    // 等待一段时间让进程启动
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 设置系统代理（在单独的作用域中处理，确保锁被释放）
    println!("[VPN] 正在设置系统代理...");
    {
        let mut proxy_mgr = PROXY_MANAGER.lock().map_err(|e| {
            eprintln!("[VPN] 获取代理管理器失败: {}", e);
            format!("获取代理管理器失败: {}", e)
        })?;

        proxy_mgr
            .enable_proxy("127.0.0.1", http_port)
            .map_err(|e| {
                eprintln!("[VPN] 设置系统代理失败: {}", e);
                // 代理设置失败不应该导致整个连接失败
                // 只记录警告
                println!("[VPN] 警告: 系统代理设置失败，但 V2Ray 已启动: {}", e);
                format!("警告: 系统代理设置失败: {}", e)
            })?;
    }

    println!("[VPN] 系统代理已设置");

    // 返回更新后的连接信息（包含 PID）
    let conn_info = VPN_MANAGER.get_connection_info().ok_or_else(|| {
        eprintln!("[VPN] 获取连接信息失败");
        "获取连接信息失败".to_string()
    })?;

    println!("[VPN] 连接成功! PID: {:?}", conn_info.pid);
    println!("[VPN] HTTP代理端口: {}", conn_info.local_port);
    println!("[VPN] SOCKS代理端口: {}", conn_info.local_socks_port);
    println!("[VPN] ===============================================");
    println!("[VPN] 请确保：");
    println!("[VPN] 1. 浏览器设置使用系统代理");
    println!("[VPN] 2. 如果使用 Chrome，重启浏览器以应用代理设置");
    println!(
        "[VPN] 3. 系统代理已设置为: 127.0.0.1:{}",
        conn_info.local_port
    );
    println!("[VPN] ===============================================");

    Ok(conn_info)
}

#[tauri::command]
pub async fn disconnect_vpn_command() -> Result<(), String> {
    println!("[VPN] 开始断开连接...");

    // 获取配置 ID 用于清理
    let config_id = VPN_MANAGER.get_connection_info().map(|info| info.config_id);

    // 禁用系统代理（在单独的作用域中处理，确保锁被释放）
    println!("[VPN] 正在禁用系统代理...");
    {
        let mut proxy_mgr = PROXY_MANAGER.lock().map_err(|e| {
            eprintln!("[VPN] 获取代理管理器失败: {}", e);
            format!("获取代理管理器失败: {}", e)
        })?;

        proxy_mgr.disable_proxy().map_err(|e| {
            eprintln!("[VPN] 禁用系统代理失败: {}", e);
            format!("禁用系统代理失败: {}", e)
        })?;
    }
    println!("[VPN] 系统代理已禁用");

    // 停止 V2Ray 进程
    println!("[VPN] 正在停止 V2Ray 进程...");
    VPN_MANAGER.stop().map_err(|e| {
        eprintln!("[VPN] 停止 VPN 失败: {}", e);
        format!("停止 VPN 失败: {}", e)
    })?;
    println!("[VPN] V2Ray 进程已停止");

    // 清理配置文件
    if let Some(id) = config_id {
        println!("[VPN] 正在清理配置文件...");
        cleanup_config(&id).await.ok();
        println!("[VPN] 配置文件已清理");
    }

    println!("[VPN] 断开连接完成");
    Ok(())
}

/// 重置系统代理设置（紧急恢复）
#[tauri::command]
pub fn reset_proxy_command() -> Result<String, String> {
    println!("[VPN] 紧急重置系统代理...");

    let mut proxy_mgr = PROXY_MANAGER
        .lock()
        .map_err(|e| format!("获取代理管理器失败: {}", e))?;

    proxy_mgr
        .disable_proxy()
        .map_err(|e| format!("禁用系统代理失败: {}", e))?;

    println!("[VPN] 系统代理已重置");
    Ok("✅ 系统代理已重置，请重启浏览器".to_string())
}

#[tauri::command]
pub fn get_connection_status_command() -> Result<Option<ConnectionInfo>, String> {
    Ok(VPN_MANAGER.get_connection_info())
}

#[tauri::command]
pub fn is_connected_command() -> Result<bool, String> {
    Ok(VPN_MANAGER.is_connected())
}

// ========== V2Ray 核心管理命令 ==========

#[tauri::command]
pub async fn check_core_command() -> Result<CoreInfo, String> {
    check_core_installed()
        .await
        .map_err(|e| format!("Failed to check core: {}", e))
}

#[tauri::command]
pub async fn download_core_command(
    app: tauri::AppHandle,
    version: Option<String>,
) -> Result<String, String> {
    let progress_callback = move |progress: DownloadProgress| {
        let _ = app.emit("core-download-progress", progress);
    };

    let archive_path = download_core(version, progress_callback)
        .await
        .map_err(|e| format!("Failed to download core: {}", e))?;

    crate::core_manager::extract_core(&archive_path)
        .await
        .map_err(|e| format!("Failed to extract core: {}", e))?;

    let exe_path = crate::core_manager::get_executable_path()
        .await
        .map_err(|e| format!("Failed to get executable path: {}", e))?;

    Ok(exe_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn remove_core_command() -> Result<(), String> {
    remove_core()
        .await
        .map_err(|e| format!("Failed to remove core: {}", e))
}

/// 设置代理模式
#[tauri::command]
pub fn set_proxy_mode_command(mode: String) -> Result<(), String> {
    let proxy_mode = match mode.as_str() {
        "global" => ProxyMode::Global,
        "pac" => ProxyMode::Pac,
        "direct" => ProxyMode::Direct,
        _ => return Err("无效的代理模式".to_string()),
    };

    let mut manager = PROXY_MANAGER
        .lock()
        .map_err(|e| format!("无法获取代理管理器: {}", e))?;

    manager.set_mode(proxy_mode.clone());

    // 保存代理模式到配置
    save_proxy_mode(&mode).map_err(|e| format!("保存代理模式失败: {}", e))?;

    Ok(())
}

/// 获取当前代理模式
#[tauri::command]
pub fn get_proxy_mode_command() -> Result<String, String> {
    // 先尝试从配置文件读取
    match load_proxy_mode() {
        Ok(mode) => Ok(mode),
        Err(_) => {
            // 如果读取失败，返回默认值
            let manager = PROXY_MANAGER
                .lock()
                .map_err(|e| format!("无法获取代理管理器: {}", e))?;

            let mode = match manager.get_mode() {
                ProxyMode::Global => "global",
                ProxyMode::Pac => "pac",
                ProxyMode::Direct => "direct",
            };

            Ok(mode.to_string())
        }
    }
}

/// 手动更新PAC文件
#[tauri::command]
pub fn update_pac_command() -> Result<String, String> {
    let mut manager = PROXY_MANAGER
        .lock()
        .map_err(|e| format!("无法获取代理管理器: {}", e))?;

    // 使用默认的本地代理地址
    manager
        .update_pac_file("127.0.0.1", 10808)
        .map_err(|e| format!("更新PAC文件失败: {}", e))?;

    Ok("PAC 文件已更新".to_string())
}

/// 下载GFWList
#[tauri::command]
pub async fn download_gfwlist_command() -> Result<String, String> {
    use crate::proxy_manager::ProxyManager;

    ProxyManager::download_gfwlist()
        .await
        .map_err(|e| format!("下载GFWList失败: {}", e))
}

/// 测试代理连接
#[tauri::command]
pub async fn test_proxy_command() -> Result<String, String> {
    println!("[Proxy Test] 开始测试代理连接...");

    // 获取当前连接信息
    let conn_info = VPN_MANAGER
        .get_connection_info()
        .ok_or_else(|| "未连接到 VPN".to_string())?;

    println!("[Proxy Test] 使用端口: {}", conn_info.local_port);

    // 测试通过 HTTP 代理访问 Google
    let proxy_url = format!("http://127.0.0.1:{}", conn_info.local_port);
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::http(&proxy_url).map_err(|e| format!("创建代理客户端失败: {}", e))?)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("构建HTTP客户端失败: {}", e))?;

    match client.get("https://www.google.com").send().await {
        Ok(response) => {
            let status = response.status();
            println!("[Proxy Test] 成功! HTTP状态: {}", status);
            Ok(format!("✅ 代理连接正常！HTTP状态: {}", status))
        }
        Err(e) => {
            eprintln!("[Proxy Test] 失败: {}", e);
            Err(format!("❌ 代理测试失败: {}\n\n请检查：\n1. V2Ray 进程是否正常运行\n2. 服务器配置是否正确\n3. 网络连接是否正常", e))
        }
    }
}
