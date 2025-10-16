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
        println!("[VpnManager] 开始启动 V2Ray 进程");

        // 检查是否已有连接
        if self.is_connected() {
            eprintln!("[VpnManager] 已有活动连接");
            return Err(anyhow!("已有活动连接，请先断开"));
        }

        // 查找 v2ray 可执行文件
        println!("[VpnManager] 查找 V2Ray 可执行文件...");
        let v2ray_path = self.find_v2ray_executable()?;
        println!("[VpnManager] 找到 V2Ray: {}", v2ray_path);

        // 启动进程
        println!("[VpnManager] 启动进程，配置文件: {}", config_path);
        let child = Command::new(&v2ray_path)
            .arg("run")
            .arg("-c")
            .arg(config_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                eprintln!("[VpnManager] 启动进程失败: {}", e);
                anyhow!("启动 V2Ray 失败: {}", e)
            })?;

        let pid = child.id();
        println!("[VpnManager] 进程已启动，PID: {}", pid);

        // 保存进程信息（使用更安全的错误处理）
        println!("[VpnManager] 获取进程锁...");
        let mut process = self.process.lock().map_err(|e| {
            eprintln!("[VpnManager] 获取进程锁失败: {}", e);
            anyhow!("获取进程锁失败: {}", e)
        })?;
        *process = Some(child);
        println!("[VpnManager] 进程已保存");

        println!("[VpnManager] 获取连接信息锁...");
        let mut conn_info = self.connection_info.lock().map_err(|e| {
            eprintln!("[VpnManager] 获取连接信息锁失败: {}", e);
            anyhow!("获取连接信息锁失败: {}", e)
        })?;
        let mut info = info;
        info.pid = Some(pid);
        *conn_info = Some(info);
        println!("[VpnManager] 连接信息已保存");

        println!("[VpnManager] V2Ray 启动完成");
        Ok(())
    }

    // 停止 V2Ray 进程
    pub fn stop(&self) -> Result<()> {
        println!("[VpnManager] 停止 V2Ray 进程");

        let mut process = self.process.lock().map_err(|e| {
            eprintln!("[VpnManager] 获取进程锁失败: {}", e);
            anyhow!("获取进程锁失败: {}", e)
        })?;

        if let Some(mut child) = process.take() {
            println!("[VpnManager] 结束进程...");
            child.kill().map_err(|e| {
                eprintln!("[VpnManager] 停止进程失败: {}", e);
                anyhow!("停止进程失败: {}", e)
            })?;
            child.wait().ok();
            println!("[VpnManager] 进程已结束");
        } else {
            println!("[VpnManager] 没有运行中的进程");
        }

        let mut conn_info = self.connection_info.lock().map_err(|e| {
            eprintln!("[VpnManager] 获取连接信息锁失败: {}", e);
            anyhow!("获取连接信息锁失败: {}", e)
        })?;
        *conn_info = None;

        println!("[VpnManager] 停止完成");
        Ok(())
    }

    // 检查是否已连接
    pub fn is_connected(&self) -> bool {
        match self.process.lock() {
            Ok(process) => process.is_some(),
            Err(e) => {
                eprintln!("[VpnManager] 检查连接状态失败: {}", e);
                false
            }
        }
    }

    // 获取连接信息
    pub fn get_connection_info(&self) -> Option<ConnectionInfo> {
        match self.connection_info.lock() {
            Ok(conn_info) => conn_info.clone(),
            Err(e) => {
                eprintln!("[VpnManager] 获取连接信息失败: {}", e);
                None
            }
        }
    }

    // 查找 V2Ray 可执行文件
    fn find_v2ray_executable(&self) -> Result<String> {
        use std::path::PathBuf;

        // 1. 优先检查核心管理下载的位置（相对于应用程序的 v2ray-core 目录）
        let core_dir = PathBuf::from("v2ray-core");

        #[cfg(target_os = "windows")]
        let core_exe = core_dir.join("v2ray.exe");

        #[cfg(not(target_os = "windows"))]
        let core_exe = core_dir.join("v2ray");

        if core_exe.exists() {
            return Ok(core_exe.to_string_lossy().to_string());
        }

        // 2. 检查当前工作目录
        let candidates = vec![
            "v2ray.exe", // Windows
            "v2ray",     // Linux/Mac
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

        // 3. 检查系统常见安装位置
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

        // 4. 未找到，提示用户使用核心管理功能
        Err(anyhow!(
            "未找到 V2Ray 核心程序。\n\
             \n\
             请通过以下方式安装：\n\
             1. 点击主界面的「核心管理」按钮\n\
             2. 在弹出的对话框中点击「下载 V2Ray 核心」\n\
             3. 等待下载完成后即可使用\n\
             \n\
             或者手动下载：\n\
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
