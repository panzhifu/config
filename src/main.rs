// 全平台配置脚本
use anyhow::{Context, Result};
use clap::Parser;
use std::env;
use std::process::Command;

// 命令行参数
#[derive(Parser)]
#[command(name = "config", about = "全平台配置工具", version)]
struct Cli {}
fn main() {}
