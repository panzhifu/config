use crate::utils::{self, FileMapping};
use anyhow::{Context, Result};

/// 更新单个文件映射
fn update_file_mapping(mapping: &FileMapping, target_os: &str) -> Result<()> {
    if !mapping.platforms.is_empty() && !mapping.platforms.contains(&target_os.to_string()) {
        println!("跳过（平台不匹配）: {}", mapping.source);
        return Ok(());
    }

    let src = utils::expand_path(&mapping.source);
    let dst = utils::expand_path(&mapping.target);

    if !src.exists() {
        println!("警告: 源文件不存在，跳过: {}", src.display());
        return Ok(());
    }

    println!("处理: {} -> {}", src.display(), dst.display());

    match mapping.strategy.as_str() {
        "copy" => {
            utils::copy_recursive(&src, &dst)?;
            println!("  复制完成");
        }
        "symlink" => {
            utils::symlink_recursive(&src, &dst)?;
            println!("  符号链接创建完成");
        }
        other => {
            anyhow::bail!("未知的策略: {}", other);
        }
    }
    Ok(())
}

pub fn cmd_update(config_source: &str) -> Result<()> {
    let target_os = utils::current_os();
    println!("目标系统: {} (自动检测)", target_os);

    let content = utils::read_config_source(config_source)?;
    let config: utils::UpdateConfig =
        toml::from_str(&content).with_context(|| format!("解析配置文件失败: {}", config_source))?;

    for mapping in &config.files {
        if !mapping.platforms.is_empty() && !mapping.platforms.contains(&target_os.to_string()) {
            println!("跳过（平台不匹配）: {}", mapping.source);
            continue;
        }
        update_file_mapping(mapping, target_os)?;
    }

    println!("更新完成。");
    Ok(())
}
