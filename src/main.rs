use anyhow::Result;
use clap::Parser;

use config::cli::{Cli, Commands};
use config::commands;

// ---------- 主函数 ----------
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Check) => commands::cmd_check(),
        Some(Commands::Update { config }) => commands::cmd_update(&config),
        None => commands::cmd_check(),
    }
}
