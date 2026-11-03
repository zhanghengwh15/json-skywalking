use super::{LogSummary, RequestLog, ServerConfig, ServerStatus, TestServer};
use std::io::Write;
use tauri::{Manager, State};

#[tauri::command]
pub async fn http_test_server_status(
    server: State<'_, TestServer>,
) -> Result<ServerStatus, String> {
    Ok(server.status().await)
}

#[tauri::command]
pub async fn http_test_server_start(
    server: State<'_, TestServer>,
    config: ServerConfig,
) -> Result<ServerStatus, String> {
    server.start(config).await
}

#[tauri::command]
pub async fn http_test_server_stop(server: State<'_, TestServer>) -> Result<ServerStatus, String> {
    server.stop().await
}

#[tauri::command]
pub fn http_test_server_logs(server: State<'_, TestServer>) -> Result<Vec<LogSummary>, String> {
    let logs = server.logs.lock().map_err(|_| "请求日志不可用")?;
    Ok(logs
        .entries
        .iter()
        .map(|entry| entry.summary.clone())
        .collect())
}

#[tauri::command]
pub fn http_test_server_log(
    server: State<'_, TestServer>,
    id: u64,
) -> Result<Option<RequestLog>, String> {
    let logs = server.logs.lock().map_err(|_| "请求日志不可用")?;
    Ok(logs
        .entries
        .iter()
        .find(|entry| entry.summary.id == id)
        .cloned())
}

#[tauri::command]
pub fn http_test_server_clear_logs(server: State<'_, TestServer>) -> Result<(), String> {
    server
        .logs
        .lock()
        .map_err(|_| "请求日志不可用")?
        .entries
        .clear();
    Ok(())
}

#[tauri::command]
pub async fn http_test_server_export_markdown(
    app: tauri::AppHandle,
    content: String,
) -> Result<String, String> {
    if content.len() > 4 * 1024 * 1024 {
        return Err("文档不能超过 4 MiB".into());
    }
    let directory = app
        .path()
        .download_dir()
        .map_err(|error| format!("无法获取下载目录：{error}"))?;
    tauri::async_runtime::spawn_blocking(move || {
        std::fs::create_dir_all(&directory)
            .map_err(|error| format!("无法创建下载目录：{error}"))?;
        let name = format!(
            "HTTP测试服务-{}-{}.md",
            chrono::Local::now().format("%Y%m%d-%H%M%S"),
            rand::random::<u32>()
        );
        let path = directory.join(name);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| format!("创建文档失败：{error}"))?;
        file.write_all(content.as_bytes())
            .map_err(|error| format!("写入文档失败：{error}"))?;
        Ok(path.to_string_lossy().into_owned())
    })
    .await
    .map_err(|error| format!("导出任务失败：{error}"))?
}
