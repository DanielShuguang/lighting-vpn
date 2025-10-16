use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

const CORE_DIR: &str = "v2ray-core";
const GITHUB_RELEASE_URL: &str = "https://github.com/v2fly/v2ray-core/releases/download";
const GITHUB_API_URL: &str = "https://api.github.com/repos/v2fly/v2ray-core/releases/latest";
const DEFAULT_VERSION: &str = "v5.16.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreInfo {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub platform: String,
    pub latest_version: Option<String>,
    pub has_update: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubRelease {
    tag_name: String,
    name: String,
    published_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
    pub percentage: f64,
}

/// 获取当前平台信息
pub fn get_platform_info() -> Result<(String, String, String)> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let (platform, archive_ext, executable_ext) = match (os, arch) {
        ("windows", "x86_64") => ("windows-64", "zip", ".exe"),
        ("windows", "x86") => ("windows-32", "zip", ".exe"),
        ("linux", "x86_64") => ("linux-64", "zip", ""),
        ("linux", "aarch64") => ("linux-arm64-v8a", "zip", ""),
        ("macos", "x86_64") => ("macos-64", "zip", ""),
        ("macos", "aarch64") => ("macos-arm64-v8a", "zip", ""),
        _ => return Err(anyhow!("不支持的平台: {} {}", os, arch)),
    };

    Ok((
        platform.to_string(),
        archive_ext.to_string(),
        executable_ext.to_string(),
    ))
}

/// 获取核心目录路径
pub async fn get_core_dir() -> Result<PathBuf> {
    let core_dir = PathBuf::from(CORE_DIR);
    if !core_dir.exists() {
        fs::create_dir_all(&core_dir).await?;
    }
    Ok(core_dir)
}

/// 获取可执行文件路径
pub async fn get_executable_path() -> Result<PathBuf> {
    let (_, _, exe_ext) = get_platform_info()?;
    let core_dir = get_core_dir().await?;
    Ok(core_dir.join(format!("v2ray{}", exe_ext)))
}

/// 获取最新版本信息
pub async fn get_latest_version() -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent("VPN-Client")
        .timeout(std::time::Duration::from_secs(10))
        .build()?;

    let response = client.get(GITHUB_API_URL).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("无法获取最新版本信息: HTTP {}", response.status()));
    }

    let release: GitHubRelease = response.json().await?;
    Ok(release.tag_name)
}

/// 比较版本号
fn compare_versions(current: &str, latest: &str) -> bool {
    // 移除 'v' 前缀
    let current = current.trim_start_matches('v');
    let latest = latest.trim_start_matches('v');

    // 简单的版本号比较
    current < latest
}

/// 检查核心是否已安装
pub async fn check_core_installed() -> Result<CoreInfo> {
    let (platform, _, _) = get_platform_info()?;
    let exe_path = get_executable_path().await?;

    // 尝试获取最新版本（失败不影响主流程）
    let latest_version = get_latest_version().await.ok();

    if exe_path.exists() {
        // 尝试获取当前版本信息
        let version = get_core_version(&exe_path).await.ok();

        // 检查是否有更新
        let has_update = if let (Some(ref current), Some(ref latest)) = (&version, &latest_version)
        {
            compare_versions(current, latest)
        } else {
            false
        };

        Ok(CoreInfo {
            installed: true,
            version,
            path: Some(exe_path.to_string_lossy().to_string()),
            platform,
            latest_version,
            has_update,
        })
    } else {
        Ok(CoreInfo {
            installed: false,
            version: None,
            path: None,
            platform,
            latest_version,
            has_update: false,
        })
    }
}

/// 获取核心版本
async fn get_core_version(exe_path: &PathBuf) -> Result<String> {
    use tokio::process::Command;

    let output = Command::new(exe_path).arg("version").output().await?;

    if output.status.success() {
        let version_output = String::from_utf8_lossy(&output.stdout);
        // 解析版本号，例如: "V2Ray 5.16.1 (V2Fly, a community-driven edition of V2Ray.)"
        if let Some(line) = version_output.lines().next() {
            if let Some(version) = line.split_whitespace().nth(1) {
                return Ok(version.to_string());
            }
        }
    }

    Err(anyhow!("无法获取版本信息"))
}

/// 构建下载 URL
pub fn build_download_url(version: &str) -> Result<String> {
    let (platform, archive_ext, _) = get_platform_info()?;

    // 构建文件名，例如: v2ray-windows-64.zip
    let filename = format!("v2ray-{}.{}", platform, archive_ext);

    // 完整 URL
    let url = format!("{}/{}/{}", GITHUB_RELEASE_URL, version, filename);

    Ok(url)
}

/// 下载 V2Ray 核心
pub async fn download_core(
    version: Option<String>,
    progress_callback: impl Fn(DownloadProgress) + Send + 'static,
) -> Result<PathBuf> {
    // 如果没有指定版本，尝试获取最新版本
    let version = if let Some(v) = version {
        v
    } else {
        // 尝试获取最新版本，如果失败则使用默认版本
        get_latest_version()
            .await
            .unwrap_or_else(|_| DEFAULT_VERSION.to_string())
    };

    let url = build_download_url(&version)?;
    let core_dir = get_core_dir().await?;

    // 下载到临时文件
    let (_, archive_ext, _) = get_platform_info()?;
    let temp_file = core_dir.join(format!("v2ray-core-temp.{}", archive_ext));

    // 发送 HTTP 请求
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("下载失败: HTTP {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);

    // 创建临时文件
    let mut file = fs::File::create(&temp_file).await?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;

        downloaded += chunk.len() as u64;
        let percentage = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        progress_callback(DownloadProgress {
            downloaded,
            total: total_size,
            percentage,
        });
    }

    file.flush().await?;
    drop(file);

    Ok(temp_file)
}

/// 解压下载的文件
pub async fn extract_core(archive_path: &PathBuf) -> Result<()> {
    let core_dir = get_core_dir().await?;

    #[cfg(target_os = "windows")]
    {
        extract_zip(archive_path, &core_dir).await?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        extract_zip(archive_path, &core_dir).await?;
    }

    // 删除临时文件
    fs::remove_file(archive_path).await?;

    // 设置可执行权限（Unix 系统）
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let exe_path = get_executable_path().await?;
        let mut perms = fs::metadata(&exe_path).await?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&exe_path, perms).await?;
    }

    Ok(())
}

/// 解压 ZIP 文件
async fn extract_zip(archive_path: &PathBuf, target_dir: &PathBuf) -> Result<()> {
    use std::io::Cursor;

    let archive_bytes = fs::read(archive_path).await?;
    let cursor = Cursor::new(archive_bytes);

    let mut archive = zip::ZipArchive::new(cursor)?;

    // 先提取所有文件信息到 Vec，避免在 await 时持有 ZipFile
    let mut files_to_extract = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name().to_string();
        let is_dir = file_name.ends_with('/');

        let buffer = if !is_dir {
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut buf)?;
            Some(buf)
        } else {
            None
        };

        files_to_extract.push((file_name, is_dir, buffer));
    }

    // 现在可以安全地使用 async 操作
    for (file_name, is_dir, buffer) in files_to_extract {
        let outpath = target_dir.join(&file_name);

        if is_dir {
            fs::create_dir_all(&outpath).await?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).await?;
            }

            if let Some(data) = buffer {
                fs::write(&outpath, data).await?;
            }
        }
    }

    Ok(())
}

/// 删除核心文件
pub async fn remove_core() -> Result<()> {
    let core_dir = get_core_dir().await?;

    // 如果目录存在，删除整个目录
    if core_dir.exists() {
        fs::remove_dir_all(&core_dir).await?;
    }

    Ok(())
}
