use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

// ---------- 平台检测 ----------
pub fn current_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}

// ---------- 路径展开 ----------
/// 将字符串中的 $HOME、%USERPROFILE% 等替换为实际路径
pub fn expand_path(path_str: &str) -> PathBuf {
    let home = dirs::home_dir().expect("无法获取家目录");
    let result = if cfg!(windows) {
        path_str.replace("%USERPROFILE%", &home.to_string_lossy())
    } else {
        path_str.replace("$HOME", &home.to_string_lossy())
    };
    PathBuf::from(result)
}

// ---------- 文件操作 ----------
/// 复制文件或目录（递归）
pub fn copy_recursive(src: &Path, dst: &Path) -> Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());
            if src_path.is_dir() {
                copy_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
    }
    Ok(())
}

/// 创建符号链接（跨平台）
pub fn symlink_recursive(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() || dst.is_symlink() {
        println!("  跳过已存在: {}", dst.display());
        return Ok(());
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(src, dst)?;
    #[cfg(windows)]
    {
        if src.is_dir() {
            std::os::windows::fs::symlink_dir(src, dst)?;
        } else {
            std::os::windows::fs::symlink_file(src, dst)?;
        }
    }
    Ok(())
}

// ---------- 配置读取 ----------
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpdateConfig {
    pub files: Vec<FileMapping>,
}

#[derive(Debug, Deserialize)]
pub struct FileMapping {
    pub source: String,
    pub target: String,
    #[serde(default = "default_strategy")]
    pub strategy: String,
    #[serde(default)]
    pub platforms: Vec<String>,
}

fn default_strategy() -> String {
    "copy".to_string()
}

/// 读取配置内容（支持本地路径和 HTTP/HTTPS URL）
pub fn read_config_source(source: &str) -> Result<String> {
    if source.starts_with("http://") || source.starts_with("https://") {
        println!("从网络下载配置: {}", source);
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        let response = client.get(source).send()?;
        if !response.status().is_success() {
            anyhow::bail!("HTTP 错误: {}", response.status());
        }
        let text = response.text()?;
        Ok(text)
    } else {
        let path = Path::new(source);
        if !path.exists() {
            anyhow::bail!("配置文件不存在: {}", path.display());
        }
        let content = fs::read_to_string(path)
            .with_context(|| format!("读取配置文件失败: {}", path.display()))?;
        Ok(content)
    }
}
