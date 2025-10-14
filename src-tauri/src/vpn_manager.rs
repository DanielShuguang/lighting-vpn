use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub config_id: String,
    pub config_name: String,
    pub local_port: u16,
    pub local_socks_port: u16,
    pub pid: Option<u32>,
}

pub struct VpnManager {
    process: Arc<Mutex<Option<Child>>>,
    connection_info: Arc<Mutex<Option<ConnectionInfo>>>,
}

impl VpnManager {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            connection_info: Arc::new(Mutex::new(None)),
        }
    }

    // 启动 V2Ray 进程
    pub async fn start_v2ray(&self, config_path: &str, info: ConnectionInfo) -> Result<()> {
        // 检查是否已有连接
        if self.is_connected() {
            return Err(anyhow!("已有活动连接，请先断开"));
        }

        // 查找 v2ray 可执行文件
        let v2ray_path = self.find_v2ray_executable()?;

        // 启动进程
        let child = Command::new(v2ray_path)
            .arg("run")
            .arg("-c")
            .arg(config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anyhow!("启动 V2Ray 失败: {}", e))?;

        let pid = child.id();

        // 保存进程信息
        let mut process = self.process.lock().unwrap();
        *process = Some(child);

        let mut conn_info = self.connection_info.lock().unwrap();
        let mut info = info;
        info.pid = Some(pid);
        *conn_info = Some(info);

        Ok(())
    }

    // 停止 V2Ray 进程
    pub fn stop(&self) -> Result<()> {
        let mut process = self.process.lock().unwrap();

        if let Some(mut child) = process.take() {
            child.kill().map_err(|e| anyhow!("停止进程失败: {}", e))?;
            child.wait().ok();
        }

        let mut conn_info = self.connection_info.lock().unwrap();
        *conn_info = None;

        Ok(())
    }

    // 检查是否已连接
    pub fn is_connected(&self) -> bool {
        let process = self.process.lock().unwrap();
        process.is_some()
    }

    // 获取连接信息
    pub fn get_connection_info(&self) -> Option<ConnectionInfo> {
        let conn_info = self.connection_info.lock().unwrap();
        conn_info.clone()
    }

    // 查找 V2Ray 可执行文件
    fn find_v2ray_executable(&self) -> Result<String> {
        // 按优先级查找
        let candidates = vec![
            "v2ray.exe", // Windows，当前目录
            "v2ray",     // Linux/Mac，当前目录
            "./v2ray.exe",
            "./v2ray",
            "xray.exe", // Xray 作为替代
            "xray",
            "./xray.exe",
            "./xray",
        ];

        for candidate in candidates {
            if let Ok(_) = which::which(candidate) {
                return Ok(candidate.to_string());
            }
        }

        // 尝试在常见位置查找
        #[cfg(target_os = "windows")]
        let common_paths = vec![
            "C:\\Program Files\\v2ray\\v2ray.exe",
            "C:\\Program Files\\xray\\xray.exe",
        ];

        #[cfg(not(target_os = "windows"))]
        let common_paths = vec![
            "/usr/local/bin/v2ray",
            "/usr/bin/v2ray",
            "/usr/local/bin/xray",
            "/usr/bin/xray",
        ];

        for path in common_paths {
            if std::path::Path::new(path).exists() {
                return Ok(path.to_string());
            }
        }

        Err(anyhow!(
            "未找到 V2Ray/Xray 可执行文件。\n\
             请确保已安装 V2Ray 或 Xray，或将可执行文件放在程序目录下。\n\
             下载地址：\n\
             - V2Ray: https://github.com/v2fly/v2ray-core/releases\n\
             - Xray: https://github.com/XTLS/Xray-core/releases"
        ))
    }
}

impl Drop for VpnManager {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

// 全局 VPN 管理器实例
lazy_static::lazy_static! {
    pub static ref VPN_MANAGER: VpnManager = VpnManager::new();
}
