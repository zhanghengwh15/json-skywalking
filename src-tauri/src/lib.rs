#![cfg_attr(not(debug_assertions), windows_subsystem = "console")]

// 引入模块
mod clipboard;
mod store;
mod tray;

// 重新导出需要的类型
pub use store::{ParseRecord, SqlHistoryItem, JsonHistoryItem};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|_app, _shortcut, _event| {
                    println!("[GlobalShortcut] 快捷键被触发");
                    let handle = _app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(e) = clipboard::process_clipboard_content(handle).await {
                            eprintln!("[GlobalShortcut] 处理剪贴板内容出错: {}", e);
                        } else {
                            println!("[GlobalShortcut] 剪贴板内容处理完成");
                        }
                    });
                })
                .build()
        )
        .invoke_handler(tauri::generate_handler![
            greet,
            // clipboard commands
            clipboard::get_clipboard,
            clipboard::global_clipboard_shortcut,
            clipboard::process_clipboard_content,
            // store commands
            store::save_parse_record,
            store::get_parse_records,
            store::delete_parse_record,
            store::clear_parse_records,
            store::load_sql_history,
            store::save_sql_history,
            store::load_json_history,
            store::save_json_history
        ])
        .on_window_event(|window, event| {
            tray::handle_window_event(window, event);
        })
        .setup(|app| {
            // 设置系统托盘
            if let Err(e) = tray::setup_tray(app) {
                eprintln!("[Tray] 设置系统托盘失败: {}", e);
            }

            // 设置全局快捷键
            #[cfg(desktop)]
            tray::setup_global_shortcut(app);

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            tray::handle_run_event(app_handle, &event);
        });
}