mod commands;
mod core_manager;
mod network_test;
mod proxy_manager;
mod storage;
mod subscription;
mod v2ray_config;
mod vpn_config;
mod vpn_manager;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 设置 panic hook 以捕获崩溃信息
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("========== 应用 Panic ==========");
        eprintln!("{}", panic_info);
        if let Some(location) = panic_info.location() {
            eprintln!(
                "位置: {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }
        eprintln!("==============================");
    }));

    println!("[App] 应用启动");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            // 配置管理命令
            load_configs_command,
            save_configs_command,
            parse_config_url_command,
            export_config_command,
            // 订阅管理命令
            load_subscriptions_command,
            add_subscription_command,
            update_subscription_command,
            delete_subscription_command,
            refresh_subscription_command,
            // 网络测试命令
            test_connection_command,
            test_latency_command,
            test_http_connection_command,
            batch_test_connections_command,
            batch_test_latencies_command,
            // VPN 连接命令
            connect_vpn_command,
            disconnect_vpn_command,
            get_connection_status_command,
            is_connected_command,
            // V2Ray 核心管理命令
            check_core_command,
            download_core_command,
            remove_core_command,
            // 代理模式命令
            set_proxy_mode_command,
            get_proxy_mode_command,
            update_pac_command,
            download_gfwlist_command,
            test_proxy_command,
            reset_proxy_command,
        ])
        .setup(|_app| {
            println!("[App] Tauri 设置完成");
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("构建 Tauri 应用失败")
        .run(|_app_handle, event| {
            match event {
                tauri::RunEvent::Exit => {
                    println!("[App] 应用正常退出");
                }
                tauri::RunEvent::ExitRequested { api, .. } => {
                    println!("[App] 收到退出请求");
                    api.prevent_exit(); // 暂时阻止退出以便查看日志
                }
                _ => {}
            }
        });
}
