use clap::{Parser, Subcommand};

// ---------- 命令行参数 ----------
#[derive(Parser)]
#[command(name = "config", about = "全平台配置工具", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 检查当前系统配置（硬件、语言环境、已安装软件）
    Check,
    /// 更新系统文件（从本地或远程配置文件同步）
    Update {
        /// 配置文件路径（本地文件或 http:// / https:// URL），默认 ./update.toml
        #[arg(long, default_value = "update.toml")]
        config: String,
    },
}
