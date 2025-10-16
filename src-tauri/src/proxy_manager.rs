use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

/// 代理模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyMode {
    /// 全局代理
    Global,
    /// PAC 模式（智能分流）
    Pac,
    /// 直连模式
    Direct,
}

impl Default for ProxyMode {
    fn default() -> Self {
        ProxyMode::Pac
    }
}

pub struct ProxyManager {
    original_settings: Option<ProxySettings>,
    current_mode: ProxyMode,
}

#[derive(Debug, Clone)]
struct ProxySettings {
    enabled: bool,
    server: String,
    #[allow(dead_code)]
    pac_url: Option<String>,
}

impl ProxyManager {
    pub fn new() -> Self {
        Self {
            original_settings: None,
            current_mode: ProxyMode::default(),
        }
    }

    /// 设置代理模式
    pub fn set_mode(&mut self, mode: ProxyMode) {
        self.current_mode = mode;
    }

    /// 获取当前代理模式
    pub fn get_mode(&self) -> &ProxyMode {
        &self.current_mode
    }

    /// 手动更新PAC文件
    pub fn update_pac_file(&mut self, host: &str, port: u16) -> Result<()> {
        let pac_content = self.generate_pac_content(host, port);
        let _ = self.generate_pac_url(&pac_content)?;
        Ok(())
    }

    /// 下载并更新GFWList
    pub async fn download_gfwlist() -> Result<String> {
        use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

        const GFWLIST_URL: &str =
            "https://raw.githubusercontent.com/gfwlist/gfwlist/master/gfwlist.txt";

        // 下载GFWList（Base64编码）
        let response = reqwest::get(GFWLIST_URL)
            .await
            .context("Failed to download GFWList")?;

        let base64_content = response.text().await.context("Failed to read response")?;

        // Base64解码
        let decoded = BASE64
            .decode(base64_content.trim())
            .context("Failed to decode GFWList")?;

        let content = String::from_utf8(decoded).context("GFWList is not valid UTF-8")?;

        // 保存到文件
        use std::fs;
        fs::write("gfwlist.txt", &content).context("Failed to save GFWList")?;

        // 统计规则数量
        let rule_count = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('!') && !trimmed.starts_with('[')
            })
            .count();

        Ok(format!("GFWList 下载成功，共 {} 条规则", rule_count))
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
        println!("[ProxyManager] 开始设置Windows代理: {}:{}", host, port);
        println!("[ProxyManager] 当前代理模式: {:?}", self.current_mode);

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let internet_settings = hkcu
            .open_subkey_with_flags(
                "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                KEY_READ | KEY_WRITE,
            )
            .map_err(|e| {
                eprintln!("[ProxyManager] 无法打开注册表: {}", e);
                anyhow!("无法打开注册表: {}", e)
            })?;

        println!("[ProxyManager] 注册表已打开");

        // 保存原始设置
        let original_enabled: u32 = internet_settings.get_value("ProxyEnable").unwrap_or(0);
        let original_server: String = internet_settings
            .get_value("ProxyServer")
            .unwrap_or_default();
        let original_pac_url: Option<String> = internet_settings.get_value("AutoConfigURL").ok();

        println!(
            "[ProxyManager] 原始设置 - 启用: {}, 服务器: {}, PAC: {:?}",
            original_enabled, original_server, original_pac_url
        );

        self.original_settings = Some(ProxySettings {
            enabled: original_enabled == 1,
            server: original_server,
            pac_url: original_pac_url,
        });

        // 根据代理模式设置
        match self.current_mode {
            ProxyMode::Global => {
                println!("[ProxyManager] 设置全局代理模式");
                // 全局代理模式：直接设置代理服务器
                let proxy_server = format!("{}:{}", host, port);
                internet_settings
                    .set_value("ProxyEnable", &1u32)
                    .map_err(|e| {
                        eprintln!("[ProxyManager] 设置 ProxyEnable 失败: {}", e);
                        anyhow!("设置 ProxyEnable 失败: {}", e)
                    })?;
                internet_settings
                    .set_value("ProxyServer", &proxy_server)
                    .map_err(|e| {
                        eprintln!("[ProxyManager] 设置 ProxyServer 失败: {}", e);
                        anyhow!("设置 ProxyServer 失败: {}", e)
                    })?;
                // 清除 PAC 设置
                internet_settings.delete_value("AutoConfigURL").ok();
                println!("[ProxyManager] 全局代理已设置: {}", proxy_server);
            }
            ProxyMode::Pac => {
                println!("[ProxyManager] 设置PAC模式");
                // PAC 模式：设置 PAC 文件 URL
                let pac_content = self.generate_pac_content(host, port);
                let pac_url = self.generate_pac_url(&pac_content)?;
                println!("[ProxyManager] PAC文件URL: {}", pac_url);

                internet_settings
                    .set_value("ProxyEnable", &1u32)
                    .map_err(|e| {
                        eprintln!("[ProxyManager] 设置 ProxyEnable 失败: {}", e);
                        anyhow!("设置 ProxyEnable 失败: {}", e)
                    })?;
                internet_settings
                    .set_value("AutoConfigURL", &pac_url)
                    .map_err(|e| {
                        eprintln!("[ProxyManager] 设置 AutoConfigURL 失败: {}", e);
                        anyhow!("设置 AutoConfigURL 失败: {}", e)
                    })?;
                println!("[ProxyManager] PAC模式已设置");
            }
            ProxyMode::Direct => {
                println!("[ProxyManager] 设置直连模式");
                // 直连模式：设置代理但添加绕过规则
                let proxy_server = format!("{}:{}", host, port);
                internet_settings
                    .set_value("ProxyEnable", &1u32)
                    .map_err(|e| {
                        eprintln!("[ProxyManager] 设置 ProxyEnable 失败: {}", e);
                        anyhow!("设置 ProxyEnable 失败: {}", e)
                    })?;
                internet_settings
                    .set_value("ProxyServer", &proxy_server)
                    .map_err(|e| {
                        eprintln!("[ProxyManager] 设置 ProxyServer 失败: {}", e);
                        anyhow!("设置 ProxyServer 失败: {}", e)
                    })?;
                // 设置绕过本地地址
                internet_settings
                    .set_value("ProxyOverride", &"<local>")
                    .map_err(|e| {
                        eprintln!("[ProxyManager] 设置 ProxyOverride 失败: {}", e);
                        anyhow!("设置 ProxyOverride 失败: {}", e)
                    })?;
                println!("[ProxyManager] 直连模式已设置: {}", proxy_server);
            }
        }

        // 刷新系统设置
        self.refresh_windows_proxy()?;

        println!("[ProxyManager] Windows代理设置完成");
        Ok(())
    }

    /// 生成 PAC 文件 URL
    #[cfg(target_os = "windows")]
    fn generate_pac_url(&self, pac_content: &str) -> Result<String> {
        use std::fs;

        // 获取临时目录
        let temp_dir = std::env::temp_dir();
        let pac_file = temp_dir.join("vpn-pac.pac");

        // 写入文件
        fs::write(&pac_file, pac_content).map_err(|e| anyhow!("写入 PAC 文件失败: {}", e))?;

        // 返回 file:// URL
        let pac_url = format!("file:///{}", pac_file.to_string_lossy().replace("\\", "/"));
        Ok(pac_url)
    }

    /// 生成 PAC 文件内容
    fn generate_pac_content(&self, host: &str, port: u16) -> String {
        use std::path::PathBuf;

        // 1. 尝试使用 GFWList（优先级最高）
        let gfwlist_path = PathBuf::from("gfwlist.txt");
        if gfwlist_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&gfwlist_path) {
                if let Some(pac) = self.build_pac_from_gfwlist(&content, host, port) {
                    return pac;
                }
            }
        }

        // 2. 尝试读取自定义 PAC 规则
        let custom_pac = PathBuf::from("pac_rules.txt");
        if custom_pac.exists() {
            if let Ok(content) = std::fs::read_to_string(&custom_pac) {
                return self.build_pac_with_custom_rules(&content, host, port);
            }
        }

        // 3. 使用内置的默认规则
        self.build_default_pac(host, port)
    }

    /// 从 GFWList 构建 PAC
    fn build_pac_from_gfwlist(&self, content: &str, host: &str, port: u16) -> Option<String> {
        // GFWList 格式：每行一个规则
        // 支持的规则类型：
        // - ||example.com - 域名匹配
        // - |http://example.com - URL 前缀匹配
        // - example.com - 关键字匹配
        // - @@example.com - 白名单（直连）
        // - ! 开头 - 注释

        let mut proxy_rules = Vec::new();
        let mut direct_rules = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // 跳过空行和注释
            if line.is_empty() || line.starts_with('!') || line.starts_with('[') {
                continue;
            }

            // 白名单规则（直连）
            if line.starts_with("@@") {
                let rule = &line[2..];
                if let Some(domain) = self.parse_gfwlist_rule(rule) {
                    direct_rules.push(format!("shExpMatch(host, \"*{}*\")", domain));
                }
                continue;
            }

            // 代理规则
            if let Some(domain) = self.parse_gfwlist_rule(line) {
                proxy_rules.push(format!("shExpMatch(host, \"*{}*\")", domain));
            }
        }

        if proxy_rules.is_empty() {
            return None;
        }

        let proxy_conditions = proxy_rules.join(" ||\n        ");
        let direct_conditions = if direct_rules.is_empty() {
            String::new()
        } else {
            format!(
                "    // GFWList 白名单（直连）\n    if ({}) return \"DIRECT\";\n    \n",
                direct_rules.join(" ||\n        ")
            )
        };

        Some(format!(
            r#"function FindProxyForURL(url, host) {{
    // 局域网直连
    if (isPlainHostName(host) ||
        shExpMatch(host, "*.local") ||
        isInNet(dnsResolve(host), "10.0.0.0", "255.0.0.0") ||
        isInNet(dnsResolve(host), "172.16.0.0", "255.240.0.0") ||
        isInNet(dnsResolve(host), "192.168.0.0", "255.255.0.0") ||
        isInNet(dnsResolve(host), "127.0.0.0", "255.255.255.0"))
        return "DIRECT";
    
{}    // GFWList 规则（需要代理）
    if ({}) return "PROXY {}:{}";
    
    // 默认直连
    return "DIRECT";
}}"#,
            direct_conditions, proxy_conditions, host, port
        ))
    }

    /// 解析 GFWList 规则
    fn parse_gfwlist_rule(&self, rule: &str) -> Option<String> {
        // 移除通配符和特殊字符
        let mut domain = rule.to_string();

        // 处理 ||example.com 格式
        if domain.starts_with("||") {
            domain = domain[2..].to_string();
        }

        // 处理 |http://example.com 格式
        if domain.starts_with("|http://") || domain.starts_with("|https://") {
            domain = domain[1..].to_string();
            if let Some(start) = domain.find("://") {
                domain = domain[start + 3..].to_string();
            }
        }

        // 移除路径部分，只保留域名
        if let Some(slash_pos) = domain.find('/') {
            domain = domain[..slash_pos].to_string();
        }

        // 移除端口
        if let Some(colon_pos) = domain.rfind(':') {
            if domain[colon_pos + 1..].chars().all(|c| c.is_numeric()) {
                domain = domain[..colon_pos].to_string();
            }
        }

        // 移除通配符
        domain = domain.replace('*', "");
        domain = domain.replace('^', "");

        // 验证是否为有效域名
        if domain.is_empty() || domain.contains('[') || domain.contains(']') {
            return None;
        }

        Some(domain)
    }

    /// 构建带自定义规则的 PAC
    fn build_pac_with_custom_rules(&self, rules: &str, host: &str, port: u16) -> String {
        // 解析自定义规则（每行一个域名）
        let custom_rules: Vec<String> = rules
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
            .map(|line| format!("shExpMatch(host, \"*{}\")", line.trim()))
            .collect();

        let custom_conditions = if custom_rules.is_empty() {
            String::new()
        } else {
            format!(
                "    // 自定义直连规则\n    if ({}) return \"DIRECT\";\n    \n",
                custom_rules.join(" ||\n        ")
            )
        };

        format!(
            r#"function FindProxyForURL(url, host) {{
    // 局域网直连
    if (isPlainHostName(host) ||
        shExpMatch(host, "*.local") ||
        isInNet(dnsResolve(host), "10.0.0.0", "255.0.0.0") ||
        isInNet(dnsResolve(host), "172.16.0.0",  "255.240.0.0") ||
        isInNet(dnsResolve(host), "192.168.0.0",  "255.255.0.0") ||
        isInNet(dnsResolve(host), "127.0.0.0", "255.255.255.0"))
        return "DIRECT";
    
{}    // 中国常见域名直连
    if (shExpMatch(host, "*.cn") ||
        shExpMatch(host, "*.baidu.com") ||
        shExpMatch(host, "*.qq.com") ||
        shExpMatch(host, "*.taobao.com") ||
        shExpMatch(host, "*.tmall.com") ||
        shExpMatch(host, "*.jd.com") ||
        shExpMatch(host, "*.163.com") ||
        shExpMatch(host, "*.sina.com") ||
        shExpMatch(host, "*.sohu.com") ||
        shExpMatch(host, "*.youku.com") ||
        shExpMatch(host, "*.alipay.com") ||
        shExpMatch(host, "*.weibo.com") ||
        shExpMatch(host, "*.douban.com") ||
        shExpMatch(host, "*.zhihu.com") ||
        shExpMatch(host, "*.bilibili.com"))
        return "DIRECT";
    
    // 其他走代理
    return "PROXY {}:{}";
}}"#,
            custom_conditions, host, port
        )
    }

    /// 构建默认 PAC
    fn build_default_pac(&self, host: &str, port: u16) -> String {
        format!(
            r#"function FindProxyForURL(url, host) {{
    // 局域网直连
    if (isPlainHostName(host) ||
        shExpMatch(host, "*.local") ||
        isInNet(dnsResolve(host), "10.0.0.0", "255.0.0.0") ||
        isInNet(dnsResolve(host), "172.16.0.0",  "255.240.0.0") ||
        isInNet(dnsResolve(host), "192.168.0.0",  "255.255.0.0") ||
        isInNet(dnsResolve(host), "127.0.0.0", "255.255.255.0"))
        return "DIRECT";
    
    // 中国常见域名直连
    if (shExpMatch(host, "*.cn") ||
        shExpMatch(host, "*.baidu.com") ||
        shExpMatch(host, "*.qq.com") ||
        shExpMatch(host, "*.taobao.com") ||
        shExpMatch(host, "*.tmall.com") ||
        shExpMatch(host, "*.jd.com") ||
        shExpMatch(host, "*.163.com") ||
        shExpMatch(host, "*.sina.com") ||
        shExpMatch(host, "*.sohu.com") ||
        shExpMatch(host, "*.youku.com") ||
        shExpMatch(host, "*.alipay.com") ||
        shExpMatch(host, "*.weibo.com") ||
        shExpMatch(host, "*.douban.com") ||
        shExpMatch(host, "*.zhihu.com") ||
        shExpMatch(host, "*.bilibili.com"))
        return "DIRECT";
    
    // 其他走代理
    return "PROXY {}:{}";
}}"#,
            host, port
        )
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

        println!("[ProxyManager] 正在刷新Windows代理设置...");

        unsafe {
            let result1 = InternetSetOptionW(
                ptr::null_mut(),
                INTERNET_OPTION_SETTINGS_CHANGED,
                ptr::null_mut(),
                0,
            );

            if result1 == 0 {
                eprintln!("[ProxyManager] 警告: INTERNET_OPTION_SETTINGS_CHANGED 调用失败");
            } else {
                println!("[ProxyManager] INTERNET_OPTION_SETTINGS_CHANGED 成功");
            }

            let result2 =
                InternetSetOptionW(ptr::null_mut(), INTERNET_OPTION_REFRESH, ptr::null_mut(), 0);

            if result2 == 0 {
                eprintln!("[ProxyManager] 警告: INTERNET_OPTION_REFRESH 调用失败");
            } else {
                println!("[ProxyManager] INTERNET_OPTION_REFRESH 成功");
            }
        }

        println!("[ProxyManager] Windows代理设置刷新完成");
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
