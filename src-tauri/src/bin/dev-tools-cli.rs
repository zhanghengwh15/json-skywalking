//! 控制台子系统的命令行入口（专门给 PowerShell / cmd 用）。
//!
//! 与 `dev-tools.exe` 共用 `tauri_app_lib::cli` 里的命令定义，但本二进制不带
//! `windows_subsystem = "windows"`，默认走 console 子系统，PowerShell 会等它结束
//! 并接管 stdout，输出能正常显示。
//!
//! GUI 由 `dev-tools.exe` 负责，这里不再回退到启动窗口。

use clap::{CommandFactory, Parser};
use tauri_app_lib::cli::{dispatch, Cli};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(command) => dispatch(command),
        None => {
            // 不带子命令时直接打印帮助，避免使用者误以为程序卡住。
            let _ = Cli::command().print_help();
            println!();
        }
    }
}
