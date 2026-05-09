// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use tauri_app_lib::cli::{dispatch, Cli};

fn main() {
    #[cfg(windows)]
    {
        extern "system" {
            fn AttachConsole(dwProcessId: u32) -> i32;
        }
        const ATTACH_PARENT_PROCESS: u32 = u32::MAX;
        // SAFETY: AttachConsole 是标准 Windows API。ATTACH_PARENT_PROCESS (-1) 表示
        // 附加到启动本进程的父进程控制台。如果父进程没有控制台或已经附加，函数返回
        // 错误码，这是安全的，不会影响程序继续运行。GUI 模式双击启动时没有父控制台，
        // 调用会静默失败，不会弹出额外窗口。
        unsafe {
            let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }

    let cli = Cli::parse();

    if let Some(command) = cli.command {
        dispatch(command);
        return;
    }

    tauri_app_lib::run();
}
