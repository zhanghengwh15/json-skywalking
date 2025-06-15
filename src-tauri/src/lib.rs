#![cfg_attr(not(debug_assertions), windows_subsystem = "console")]

use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
use tauri_plugin_store::StoreExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;

#[derive(Serialize, Deserialize, Clone)]
pub struct ParseRecord {
    pub id: String,
    pub timestamp: i64,
    pub request_type: String, // "GET" or "POST"
    pub url: String,
    pub json_data: Option<serde_json::Value>,
    pub curl_command: String,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SqlHistoryItem {
    sql: String,
    formatted_sql: String,
    timestamp: i64,
    hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SqlHistory {
    items: Vec<SqlHistoryItem>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_clipboard(_app_handle: tauri::AppHandle) -> Result<String, String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.get_text().map_err(|e| e.to_string())
}

#[tauri::command]
async fn global_clipboard_shortcut(app: tauri::AppHandle) -> Result<String, String> {
    // 先尝试获取剪贴板内容
    let clipboard_text = match get_clipboard(app.clone()) {
        Ok(text) => text,
        Err(e) => return Err(e),
    };
    
    // 检查是否为有效JSON
    if let Ok(_) = serde_json::from_str::<serde_json::Value>(&clipboard_text) {
        // 发送事件到前端，通知检测到JSON格式
        app.emit("global-clipboard-json", clipboard_text.clone())
            .map_err(|e| format!("Failed to emit event: {}", e))?;
        
        Ok(clipboard_text)
    } else {
        // 发送事件到前端，通知非JSON格式
        app.emit("global-clipboard-not-json", clipboard_text.clone())
            .map_err(|e| format!("Failed to emit event: {}", e))?;
        
        Err("剪贴板内容不是有效的JSON格式".to_string())
    }
}

#[tauri::command]
async fn process_clipboard_content(app: tauri::AppHandle) -> Result<String, String> {
    println!("[Clipboard] 开始处理剪贴板内容...");
    // 读取剪贴板
    let clipboard_text = match Clipboard::new() {
        Ok(mut clipboard) => clipboard.get_text().unwrap_or_default(),
        Err(e) => {
            eprintln!("[Clipboard] 错误: 无法创建剪贴板实例: {}", e);
            return Err("无法读取剪贴板内容".to_string());
        }
    };
    let content = clipboard_text.trim();
    println!("[Clipboard] 获取到的内容:\n---\n{}\n---", content);

    if content.is_empty() {
        println!("[Clipboard] 内容为空, 终止处理.");
        app.emit("process-clipboard-done", "剪贴板内容为空").ok();
        return Err("剪贴板内容为空".to_string());
    }

    // 判断 HTTP 还是 SQL
    let is_http = content.starts_with("GET ") || content.starts_with("POST ") || content.contains("http://") || content.contains("https://") || content.contains("http.body:");
    let is_sql = content.to_uppercase().contains("SELECT") || content.to_uppercase().contains("INSERT") || content.to_uppercase().contains("UPDATE") || content.to_uppercase().contains("DELETE") || content.contains("db.sql.parameters:");

    println!("[Clipboard] 类型判断: is_http={}, is_sql={}", is_http, is_sql);

    if is_http {
        println!("[Clipboard] 判断为 HTTP 请求.");
        // 简单判断是否已格式化（包含 curl 或多行参数）
        if content.contains("curl -X") {
            app.emit("process-clipboard-done", "HTTP请求已格式化，无需处理").ok();
            return Ok("HTTP请求已格式化，无需处理".to_string());
        }
        // 解析 HTTP
        let (curl, title) = if content.contains("http.body:") {
            // POST
            let parts: Vec<&str> = content.splitn(2, "http.body:").collect();
            let url = parts.get(0).map(|s| s.trim()).unwrap_or("");
            let json = parts.get(1).map(|s| s.trim()).unwrap_or("");
            let url_path = extract_url_path(url);
            let curl = format!("curl -X POST -H \"Accept-Language:zh-CN\" -H \"logLevel:debug\" -H \"Content-Type:application/json\" -d '{{}}' --url \"http://localhost:8080/{}\"", url_path);
            let curl = curl.replace("'{}'", &format!("'{}'", json.replace("'", "'\"'\"'")));
            (curl, format!("POST - {}", url_path))
        } else {
            // GET
            let url = content.trim();
            let url_path = extract_url_path(url);
            let curl = format!("curl -X GET -H \"Accept-Language:zh-CN\" -H \"logLevel:debug\" --url \"http://localhost:8080/{}\"", url_path);
            (curl, format!("GET - {}", url_path))
        };
        // 写回剪贴板
        if let Ok(mut clipboard) = Clipboard::new() {
            clipboard.set_text(&curl).ok();
        }
        // 保存到 HTTP 历史（调用 save_parse_record）
        let record = ParseRecord {
            id: format!("{}-{}", chrono::Utc::now().timestamp_millis(), rand::random::<u32>()),
            timestamp: chrono::Utc::now().timestamp_millis(),
            request_type: if content.contains("http.body:") { "POST".to_string() } else { "GET".to_string() },
            url: content.to_string(),
            json_data: None,
            curl_command: curl.clone(),
            title: Some(title.clone()),
        };
        let store = app.store("parse_history.json").map_err(|e| format!("store error: {}", e))?;
        let mut records: Vec<ParseRecord> = store.get("records").and_then(|v| serde_json::from_value(v.clone()).ok()).unwrap_or_default();
        records.insert(0, record);
        if records.len() > 100 { records.truncate(100); }
        store.set("records".to_string(), serde_json::to_value(&records).unwrap());
        store.save().ok();
        app.emit("process-clipboard-done", "HTTP请求已格式化并写入剪贴板").ok();
        return Ok("HTTP请求已格式化并写入剪贴板".to_string());
    } else if is_sql {
        println!("[Clipboard] 判断为 SQL 请求.");
        // 简单判断是否已格式化（多行缩进/关键字）
        if content.contains("\nSELECT") || content.contains("\nFROM") {
            println!("[Clipboard] SQL 已被格式化, 跳过处理.");
            app.emit("process-clipboard-done", "SQL已格式化，无需处理").ok();
            return Ok("SQL已格式化，无需处理".to_string());
        }
        println!("[Clipboard] 准备格式化 SQL...");
        // 格式化 SQL
        let formatted = format_sql_string(content);
        // 写回剪贴板
        if let Ok(mut clipboard) = Clipboard::new() {
            clipboard.set_text(&formatted).ok();
        }
        // 保存到 SQL 历史
        let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        std::fs::create_dir_all(&app_dir).ok();
        let history_file = app_dir.join("sql_history.json");
        let mut items: Vec<SqlHistoryItem> = if history_file.exists() {
            let json_str = std::fs::read_to_string(&history_file).unwrap_or_default();
            serde_json::from_str(&json_str).unwrap_or_default()
        } else {
            Vec::new()
        };
        let hash = generate_hash(content);
        if !items.iter().any(|item| item.hash == hash) {
            items.insert(0, SqlHistoryItem {
                sql: content.to_string(),
                formatted_sql: formatted.clone(),
                timestamp: chrono::Utc::now().timestamp_millis(),
                hash,
            });
            if items.len() > 20 { items.truncate(20); }
            let json_history = SqlHistory { items: items.clone() };
            let json_str = serde_json::to_string_pretty(&json_history).unwrap();
            std::fs::write(&history_file, json_str).ok();
        }
        println!("[Clipboard] SQL 格式化完成并已写入剪贴板.");
        app.emit("process-clipboard-done", "SQL已格式化并写入剪贴板").ok();
        return Ok("SQL已格式化并写入剪贴板".to_string());
    } else {
        println!("[Clipboard] 未能识别为 HTTP 或 SQL.");
        app.emit("process-clipboard-done", "未识别为HTTP或SQL").ok();
        return Err("未识别为HTTP或SQL".to_string());
    }
}

fn extract_url_path(url: &str) -> String {
    // 简单提取路径
    if let Some(idx) = url.find("//") {
        let rest = &url[idx+2..];
        if let Some(idx2) = rest.find('/') {
            return rest[idx2+1..].to_string();
        }
    }
    url.trim_start_matches("/").to_string()
}

fn generate_hash(sql: &str) -> String {
    let mut hash = 0i64;
    for b in sql.bytes() {
        hash = ((hash << 5) - hash) + b as i64;
        hash &= 0xFFFFFFFF;
    }
    format!("{:x}", hash.abs())
}

fn format_sql_string(sql: &str) -> String {
    let mut formatted = sql.replace("SELECT", "\nSELECT")
        .replace("FROM", "\nFROM")
        .replace("WHERE", "\nWHERE")
        .replace("AND", "\n  AND")
        .replace("OR", "\n  OR")
        .replace("ORDER BY", "\nORDER BY")
        .replace("GROUP BY", "\nGROUP BY")
        .replace("HAVING", "\nHAVING")
        .replace("LIMIT", "\nLIMIT")
        .replace("JOIN", "\nJOIN")
        .replace("LEFT JOIN", "\nLEFT JOIN")
        .replace("RIGHT JOIN", "\nRIGHT JOIN")
        .replace("INNER JOIN", "\nINNER JOIN");
    if formatted.starts_with("\n") { formatted = formatted[1..].to_string(); }
    formatted
}

#[tauri::command]
async fn save_parse_record(
    app: tauri::AppHandle,
    record: ParseRecord,
) -> Result<(), String> {
    let store = app.store("parse_history.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;
    
    // 获取现有记录
    let mut records: Vec<ParseRecord> = store
        .get("records")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    
    // 添加新记录到开头
    records.insert(0, record);
    
    // 限制最多保存100条记录
    if records.len() > 100 {
        records.truncate(100);
    }
    
    // 保存更新后的记录
    store.set("records".to_string(), serde_json::to_value(&records).unwrap());
    store.save()
        .map_err(|e| format!("Failed to save store: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn get_parse_records(app: tauri::AppHandle) -> Result<Vec<ParseRecord>, String> {
    let store = app.store("parse_history.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;
    
    let records: Vec<ParseRecord> = store
        .get("records")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    
    Ok(records)
}

#[tauri::command]
async fn delete_parse_record(
    app: tauri::AppHandle,
    record_id: String,
) -> Result<(), String> {
    let store = app.store("parse_history.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;
    
    let mut records: Vec<ParseRecord> = store
        .get("records")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    
    // 删除指定ID的记录
    records.retain(|r| r.id != record_id);
    
    // 保存更新后的记录
    store.set("records".to_string(), serde_json::to_value(&records).unwrap());
    store.save()
        .map_err(|e| format!("Failed to save store: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn clear_parse_records(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("parse_history.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;
    
    store.set("records".to_string(), serde_json::to_value(Vec::<ParseRecord>::new()).unwrap());
    store.save()
        .map_err(|e| format!("Failed to save store: {}", e))?;
    
    Ok(())
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
                        if let Err(e) = process_clipboard_content(handle).await {
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
            get_clipboard,
            global_clipboard_shortcut,
            process_clipboard_content,
            save_parse_record,
            get_parse_records,
            delete_parse_record,
            clear_parse_records,
        ])
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, GlobalShortcutExt};
                let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyQ);
                match app.global_shortcut().register(shortcut) {
                    Ok(_) => println!("[GlobalShortcut] 快捷键注册成功: CTRL+ALT+Q"),
                    Err(e) => eprintln!("[GlobalShortcut] 快捷键注册失败: {}", e),
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
