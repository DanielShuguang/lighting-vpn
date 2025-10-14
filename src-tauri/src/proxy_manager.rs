use anyhow::{anyhow, Result};

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

pub struct ProxyManager {
    original_settings: Option<ProxySettings>,
}

#[derive(Debug, Clone)]
struct ProxySettings {
    enabled: bool,
    server: String,
}

impl ProxyManager {
    pub fn new() -> Self {
        Self {
            original_settings: None,
        }
    }

    // 设置系统代理
    pub fn enable_proxy(&mut self, host: &str, port: u16) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            self.enable_proxy_windows(host, port)
        }

        #[cfg(target_os = "macos")]
        {
            self.enable_proxy_macos(host, port)
        }

        #[cfg(target_os = "linux")]
        {
            self.enable_proxy_linux(host, port)
        }
    }

    // 禁用系统代理
    pub fn disable_proxy(&mut self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            self.disable_proxy_windows()
        }

        #[cfg(target_os = "macos")]
        {
            self.disable_proxy_macos()
        }

        #[cfg(target_os = "linux")]
        {
            self.disable_proxy_linux()
        }
    }

    // Windows 代理设置
    #[cfg(target_os = "windows")]
    fn enable_proxy_windows(&mut self, host: &str, port: u16) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let internet_settings = hkcu
            .open_subkey_with_flags(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                KEY_READ | KEY_WRITE,
            )
            .map_err(|e| anyhow!("无法打开注册表: {}", e))?;

        // 保存原始设置
        let original_enabled: u32 = internet_settings.get_value("ProxyEnable").unwrap_or(0);
        let original_server: String = internet_settings
            .get_value("ProxyServer")
            .unwrap_or_default();

        self.original_settings = Some(ProxySettings {
            enabled: original_enabled == 1,
            server: original_server,
        });

        // 设置新代理
        let proxy_server = format!("{}:{}", host, port);
        internet_settings
            .set_value("ProxyEnable", &1u32)
            .map_err(|e| anyhow!("设置 ProxyEnable 失败: {}", e))?;
        internet_settings
            .set_value("ProxyServer", &proxy_server)
            .map_err(|e| anyhow!("设置 ProxyServer 失败: {}", e))?;

        // 刷新系统设置
        self.refresh_windows_proxy()?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn disable_proxy_windows(&mut self) -> Result<()> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let internet_settings = hkcu
            .open_subkey_with_flags(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                KEY_WRITE,
            )
            .map_err(|e| anyhow!("无法打开注册表: {}", e))?;

        // 恢复原始设置
        if let Some(original) = &self.original_settings {
            let enable_value = if original.enabled { 1u32 } else { 0u32 };
            internet_settings
                .set_value("ProxyEnable", &enable_value)
                .ok();
            if !original.server.is_empty() {
                internet_settings
                    .set_value("ProxyServer", &original.server)
                    .ok();
            }
        } else {
            // 如果没有保存原始设置，直接禁用
            internet_settings.set_value("ProxyEnable", &0u32).ok();
        }

        // 刷新系统设置
        self.refresh_windows_proxy()?;

        self.original_settings = None;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn refresh_windows_proxy(&self) -> Result<()> {
        use std::ptr;
        use winapi::um::wininet::{
            InternetSetOptionW, INTERNET_OPTION_REFRESH, INTERNET_OPTION_SETTINGS_CHANGED,
        };

        unsafe {
            InternetSetOptionW(
                ptr::null_mut(),
                INTERNET_OPTION_SETTINGS_CHANGED,
                ptr::null_mut(),
                0,
            );
            InternetSetOptionW(ptr::null_mut(), INTERNET_OPTION_REFRESH, ptr::null_mut(), 0);
        }
        Ok(())
    }

    // macOS 代理设置
    #[cfg(target_os = "macos")]
    fn enable_proxy_macos(&mut self, host: &str, port: u16) -> Result<()> {
        // 使用 networksetup 命令设置代理
        std::process::Command::new("networksetup")
            .args(&["-setwebproxy", "Wi-Fi", host, &port.to_string()])
            .output()
            .map_err(|e| anyhow!("设置 HTTP 代理失败: {}", e))?;

        std::process::Command::new("networksetup")
            .args(&["-setsecurewebproxy", "Wi-Fi", host, &port.to_string()])
            .output()
            .map_err(|e| anyhow!("设置 HTTPS 代理失败: {}", e))?;

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn disable_proxy_macos(&mut self) -> Result<()> {
        std::process::Command::new("networksetup")
            .args(&["-setwebproxystate", "Wi-Fi", "off"])
            .output()
            .ok();

        std::process::Command::new("networksetup")
            .args(&["-setsecurewebproxystate", "Wi-Fi", "off"])
            .output()
            .ok();

        Ok(())
    }

    // Linux 代理设置（通过环境变量）
    #[cfg(target_os = "linux")]
    fn enable_proxy_linux(&mut self, host: &str, port: u16) -> Result<()> {
        let proxy_url = format!("http://{}:{}", host, port);

        // 设置环境变量（这只对当前进程有效）
        std::env::set_var("http_proxy", &proxy_url);
        std::env::set_var("https_proxy", &proxy_url);
        std::env::set_var("HTTP_PROXY", &proxy_url);
        std::env::set_var("HTTPS_PROXY", &proxy_url);

        // 尝试使用 gsettings 设置 GNOME 代理
        std::process::Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy.http", "host", host])
            .output()
            .ok();

        std::process::Command::new("gsettings")
            .args(&[
                "set",
                "org.gnome.system.proxy.http",
                "port",
                &port.to_string(),
            ])
            .output()
            .ok();

        std::process::Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy", "mode", "manual"])
            .output()
            .ok();

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn disable_proxy_linux(&mut self) -> Result<()> {
        // 清除环境变量
        std::env::remove_var("http_proxy");
        std::env::remove_var("https_proxy");
        std::env::remove_var("HTTP_PROXY");
        std::env::remove_var("HTTPS_PROXY");

        // 禁用 GNOME 代理
        std::process::Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy", "mode", "none"])
            .output()
            .ok();

        Ok(())
    }
}

// 全局代理管理器
lazy_static::lazy_static! {
    pub static ref PROXY_MANAGER: std::sync::Mutex<ProxyManager> = std::sync::Mutex::new(ProxyManager::new());
}
