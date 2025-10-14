mod commands;
mod network_test;
mod storage;
mod subscription;
mod vpn_config;
mod vpn_manager;
mod proxy_manager;
mod v2ray_config;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
