pub mod commands;
pub mod db;
pub mod http;

use tauri::Manager;

use db::Db;

pub fn init(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let app_data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_data_dir)?;

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
            eprintln!("[cookie-bridge] {}", msg);

            #[cfg(desktop)]
            {
                use tauri::Emitter;
                let _ = app_handle.emit("tauri://notification", &msg);
            }
        }
    });

    Ok(())
}
