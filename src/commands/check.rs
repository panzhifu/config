use anyhow::Result;
use colored::*;
use std::env;
use std::path::Path;
use std::process::Command;
use sysinfo::{Disks, System};

// ================================================================
// 主入口
// ================================================================
pub fn cmd_check() -> Result<()> {
    step("硬件配置", Color::Cyan);
    check_hardware()?;

    step("语言环境配置", Color::Green);
    check_language_environments();

    step("已安装软件", Color::Yellow);
    check_installed_software()?;

    Ok(())
}

fn step(title: &str, c: Color) {
    println!("{}", format!("═══ {title} ═══").color(c).bold());
}

// ================================================================
// 第一步：硬件配置
// ================================================================
fn check_hardware() -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_all();
    let disks = Disks::new_with_refreshed_list();

    hdr("系统信息");
    println!("    主机名:     {}", hostname());
    println!(
        "    操作系统:   {}",
        System::name().unwrap_or_else(|| "未知".into())
    );
    println!(
        "    内核版本:   {}",
        System::kernel_version().unwrap_or_else(|| "未知".into())
    );
    println!(
        "    系统版本:   {}",
        System::os_version().unwrap_or_else(|| "未知".into())
    );
    println!("    架构:       {}", env::consts::ARCH);

    hdr("CPU");
    if let Some(cpu) = sys.cpus().first() {
        println!("    型号:       {}", cpu.name());
        println!(
            "    核心数:     {} (逻辑线程: {})",
            System::physical_core_count().unwrap_or(0),
            sys.cpus().len()
        );
        println!("    频率:       {} MHz", cpu.frequency());
    }

    hdr("内存");
    println!("    总内存:     {}", fmt_bytes(sys.total_memory()));
    println!("    已用:       {}", fmt_bytes(sys.used_memory()));
    println!("    可用:       {}", fmt_bytes(sys.free_memory()));
    if sys.total_swap() > 0 {
        println!("    虚拟内存:   {}", fmt_bytes(sys.total_swap()));
    }

    hdr("磁盘");
    for d in disks.list() {
        let total = d.total_space();
        let used = total - d.available_space();
        let pct = if total > 0 {
            used as f64 / total as f64 * 100.0
        } else {
            0.0
        };
        println!(
            "    {}  {} / {}  ({:.1}%)",
            d.name().to_string_lossy(),
            fmt_bytes(used),
            fmt_bytes(total),
            pct
        );
    }

    hdr("GPU");
    check_gpu();
    Ok(())
}

fn hdr(s: &str) {
    println!("\n  {}", format!("【{s}】").bold());
}

// ---------- GPU ----------
fn check_gpu() {
    #[cfg(target_os = "windows")]
    match run_lines(
        "wmic",
        &["path", "win32_videocontroller", "get", "name,adapterram"],
    ) {
        Some(lines) => {
            for line in lines.skip(1).filter(|l| !l.trim().is_empty()) {
                let p: Vec<_> = line.trim().split_whitespace().collect();
                if p.len() < 2 {
                    println!("    {line}");
                    continue;
                }
                let mem: u64 = p.last().unwrap_or(&"0").parse().unwrap_or(0);
                println!(
                    "    {}  ({} VRAM)",
                    p[..p.len() - 1].join(" "),
                    fmt_bytes(mem)
                );
            }
        }
        None => println!("    无法获取 GPU 信息（需要管理员权限）"),
    }

    #[cfg(target_os = "linux")]
    match run_lines("lspci", &["-vnn"]) {
        Some(lines) => {
            for line in lines.filter(|l| {
                let s = l.to_lowercase();
                s.contains("vga") || s.contains("3d") || s.contains("display")
            }) {
                println!("    {line}");
            }
        }
        None => println!("    无法获取 GPU 信息"),
    }

    #[cfg(target_os = "macos")]
    match run_lines("system_profiler", &["SPDisplaysDataType"]) {
        Some(lines) => {
            for line in lines.filter(|l| !l.trim().is_empty()) {
                println!("    {line}");
            }
        }
        None => println!("    无法获取 GPU 信息"),
    }
}

// ================================================================
// 第二步：语言环境检查
// ================================================================

macro_rules! define_languages {
    ($($v:ident => $exe:literal => $disp:literal),* $(,)?) => {
        #[derive(Clone, Copy)]
        enum Language { $($v),* }
        impl Language {
            const ALL: &[Self] = &[$(Self::$v),*];
            fn exe(self) -> &'static str { match self { $(Self::$v => $exe),* } }
            fn display(self) -> &'static str { match self { $(Self::$v => $disp),* } }
            fn version_args(self) -> &'static [&'static str] {
                if matches!(self, Self::Go) { &["version"] } else { &["--version"] }
            }
        }
    };
}

define_languages! {
    Python  => "python"  => "Python",    Python3 => "python3" => "Python3",
    Node    => "node"    => "Node.js",   Npm     => "npm"     => "npm",
    Go      => "go"      => "Go",        Rust    => "rustc"   => "Rust",
    Cargo   => "cargo"   => "Cargo",     Java    => "java"    => "Java",
    Javac   => "javac"   => "Javac",     DotNet  => "dotnet"  => ".NET",
    Ruby    => "ruby"    => "Ruby",      Perl    => "perl"    => "Perl",
    Php     => "php"     => "PHP",       Git     => "git"     => "Git",
    Docker  => "docker"  => "Docker",    Gcc     => "gcc"     => "GCC",
    Clang   => "clang"   => "Clang",     Cmake   => "cmake"   => "CMake",
    Make    => "make"    => "Make",
}

fn check_language_environments() {
    let path_dirs = env::var("PATH").unwrap_or_default();
    let total = Language::ALL.len();
    let mut found = 0;

    println!();
    for &lang in Language::ALL {
        if find_program(lang.exe(), &path_dirs).is_some() {
            found += 1;
            let ver = get_version(lang.exe(), lang.version_args());
            println!(
                "    {} {:12} {}",
                "•".green(),
                format!("{}:", lang.display()),
                if ver.is_empty() { "(已安装)" } else { &ver }
            );
        }
    }
    println!(
        "\n    已检测到 {}/{} 个语言/工具",
        found.to_string().green(),
        total
    );
}

fn get_version(exe: &str, args: &[&str]) -> String {
    let out = match Command::new(exe).args(args).output() {
        Ok(o) => o,
        Err(_) => return String::new(),
    };
    let pick = |s: &[u8]| {
        String::from_utf8_lossy(s)
            .lines()
            .next()
            .unwrap_or("")
            .trim()
            .to_string()
    };
    let v = pick(&out.stdout);
    if v.is_empty() { pick(&out.stderr) } else { v }
}

fn find_program(exe: &str, path_dirs: &str) -> Option<String> {
    #[cfg(target_os = "windows")]
    let (sep, try_exe) = (';', {
        let e = if exe.ends_with(".exe") {
            exe.into()
        } else {
            format!("{exe}.exe")
        };
        if let Some(o) = run_cmd("where", &[&e]) {
            if let Some(line) = o.lines().next().filter(|l| !l.trim().is_empty()) {
                return Some(line.into());
            }
        }
        e
    });
    #[cfg(not(target_os = "windows"))]
    let (sep, try_exe) = (':', {
        if let Some(o) = run_cmd("which", &[exe]) {
            if let Some(line) = o.lines().next().filter(|l| !l.trim().is_empty()) {
                return Some(line.into());
            }
        }
        exe.into()
    });

    path_dirs
        .split(sep)
        .map(|d| Path::new(d).join(&try_exe))
        .find(|p| p.exists())
        .map(|p| p.to_string_lossy().into())
}

/// 运行命令并返回 stdout 文本
fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
}

/// 运行命令并返回 stdout 行迭代器
fn run_lines(cmd: &str, args: &[&str]) -> Option<impl Iterator<Item = String>> {
    run_cmd(cmd, args).map(|s| {
        s.lines()
            .map(str::to_string)
            .collect::<Vec<_>>()
            .into_iter()
    })
}

// ================================================================
// 第三步：已安装软件
// ================================================================
fn check_installed_software() -> Result<()> {
    #[cfg(target_os = "windows")]
    check_installed_software_windows()?;
    #[cfg(target_os = "linux")]
    check_installed_software_linux()?;
    #[cfg(target_os = "macos")]
    check_installed_software_macos()?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn check_installed_software_windows() -> Result<()> {
    println!();
    match Command::new("winget").args(["list"]).output() {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stdout);
            let is_header =
                |l: &&str| l.starts_with("---") || l.starts_with("Name") || l.starts_with("名前");
            let items: Vec<_> = text
                .lines()
                .skip_while(|l| !is_header(l))
                .filter(|l| !is_header(l) && !l.trim().is_empty())
                .collect();
            for line in &items {
                let p: Vec<_> = line.split_whitespace().collect();
                println!(
                    "    {}  {}",
                    p.first().map_or("", |s| s).cyan(),
                    p.get(1..).map_or(String::new(), |s| s.join(" "))
                );
            }
            println!(
                "\n    共检测到 {} 个通过 winget 管理的软件包",
                items.len().to_string().yellow()
            );
        }
        _ => println!(
            "    winget 未安装，扫描常见目录...\n{}\n    提示: 安装 winget 可获得更完整的软件列表。",
            scan_dirs()
        ),
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn scan_dirs() -> String {
    let mut s = String::new();
    for (dir, label) in &[
        (r"C:\Program Files", "系统范围"),
        (r"C:\Program Files (x86)", "系统范围 (32位)"),
    ] {
        s.push_str(&format!("    [{label}]\n"));
        if let Ok(e) = std::fs::read_dir(dir) {
            for f in e.flatten() {
                s.push_str(&format!("      - {}\n", f.file_name().to_string_lossy()));
            }
        }
    }
    s
}

#[cfg(target_os = "linux")]
fn check_installed_software_linux() -> Result<()> {
    println!();
    for (mgr, cmd) in &[
        ("dpkg", &["dpkg", "-l"][..]),
        ("rpm", &["rpm", "-qa"][..]),
        ("pacman", &["pacman", "-Q"][..]),
        ("apk", &["apk", "info"][..]),
    ] {
        if let Ok(o) = Command::new(cmd[0])
            .args(&cmd[1..])
            .output()
            .filter(|o| o.status.success())
        {
            let text = String::from_utf8_lossy(&o.stdout);
            let count = text.lines().count();
            println!("    包管理器: {mgr}  ({} 个包)", count.to_string().yellow());
            for line in text.lines().take(50).filter(|l| !l.trim().is_empty()) {
                println!("      {}", line.trim());
            }
            if count > 50 {
                println!("      ... 以及另外 {} 个包", count - 50);
            }
            return Ok(());
        }
    }
    println!("    未检测到已知的包管理器");
    Ok(())
}

#[cfg(target_os = "macos")]
fn check_installed_software_macos() -> Result<()> {
    println!();
    if let Ok(o) = Command::new("brew")
        .args(["list", "--formula"])
        .output()
        .filter(|o| o.status.success())
    {
        let text = String::from_utf8_lossy(&o.stdout);
        let count = text.lines().count();
        println!("    Homebrew 公式: {} 个", count.to_string().yellow());
        for line in text.lines().take(50) {
            println!("      {}", line.trim());
        }
        if count > 50 {
            println!("      ... 以及另外 {} 个", count - 50);
        }
    } else {
        println!("    Homebrew 未安装");
    }

    println!("\n    [Mac App Store / 系统应用]");
    if let Ok(o) = Command::new("ls").args(["/Applications"]).output() {
        for line in String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| l.ends_with(".app"))
        {
            println!("      {}", line.trim_end_matches(".app"));
        }
    }
    Ok(())
}

// ================================================================
// 辅助函数
// ================================================================
fn hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "无法获取".into())
}

fn fmt_bytes(bytes: u64) -> String {
    const U: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let (mut v, mut i) = (bytes as f64, 0);
    while v >= 1024.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    format!("{v:.2} {}", U[i])
}
