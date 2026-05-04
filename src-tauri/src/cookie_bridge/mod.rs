pub mod commands;
pub mod db;
pub mod http;

use log::LevelFilter;
use simplelog::{CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger};
use std::fs::OpenOptions;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::Manager;

static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_debug_mode(enabled: bool) {
    DEBUG_MODE.store(enabled, Ordering::Relaxed);
    log::info!("[CookieBridge] debug 模式切换: {}", enabled);
}

pub fn is_debug_mode() -> bool {
    DEBUG_MODE.load(Ordering::Relaxed)
}

use db::Db;

pub fn init(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;

    let log_path = app_data_dir.join("cookie-bridge.log");
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    let config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .build();

    let _ = CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            config.clone(),
            TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ),
        WriteLogger::new(LevelFilter::Info, config, log_file),
    ]);

    log::info!("[CookieBridge] 日志初始化完成: {:?}", log_path);

    let db_path = app_data_dir.join("data.db");
    let db = Db::open(&db_path)?;
    app.manage(db.clone());

    let app_handle = app.handle().clone();
    let db_for_http = db.clone();

    tauri::async_runtime::spawn(async move {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 8765));
        let state = http::AppState {
            db: db_for_http,
            app_handle: app_handle.clone(),
        };

        if let Err(e) = http::serve(addr, state).await {
            let msg = format!("Cookie 桥不可用：端口 8765 被占 ({e})");
            log::error!("[cookie-bridge] {}", msg);

            #[cfg(desktop)]
            {
                use tauri::Emitter;
                let _ = app_handle.emit("tauri://notification", &msg);
            }
        }
    });

    Ok(())
}
