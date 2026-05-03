use arboard::Clipboard;
// use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use crate::desktop::store::ParseRecord;

/// 获取剪贴板内容
#[tauri::command]
pub fn get_clipboard(_app_handle: tauri::AppHandle) -> Result<String, String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.get_text().map_err(|e| e.to_string())
}

/// 全局剪贴板快捷键处理
#[tauri::command]
pub async fn global_clipboard_shortcut(app: tauri::AppHandle) -> Result<String, String> {
    let clipboard_text = match get_clipboard(app.clone()) {
        Ok(text) => text,
        Err(e) => return Err(e),
    };
    
    if let Ok(_) = serde_json::from_str::<serde_json::Value>(&clipboard_text) {
        app.emit("global-clipboard-json", clipboard_text.clone())
            .map_err(|e| format!("Failed to emit event: {}", e))?;
        Ok(clipboard_text)
    } else {
        app.emit("global-clipboard-not-json", clipboard_text.clone())
            .map_err(|e| format!("Failed to emit event: {}", e))?;
        Err("剪贴板内容不是有效的JSON格式".to_string())
    }
}

/// 处理剪贴板内容的主要函数
#[tauri::command]
pub async fn process_clipboard_content(app: tauri::AppHandle) -> Result<String, String> {
    println!("[Clipboard] 开始处理剪贴板内容...");
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

    let is_http = content.starts_with("GET ") || content.starts_with("POST ") || content.contains("http://") || content.contains("https://") || content.contains("http.body:");
    let is_sql = content.to_uppercase().contains("SELECT") || content.to_uppercase().contains("INSERT") || content.to_uppercase().contains("UPDATE") || content.to_uppercase().contains("DELETE") || content.contains("db.sql.parameters:");

    println!("[Clipboard] 类型判断: is_http={}, is_sql={}", is_http, is_sql);

    if is_http {
        handle_http_content(&app, content).await
    } else if is_sql {
        handle_sql_content(&app, content).await
    } else {
        println!("[Clipboard] 未能识别为 HTTP 或 SQL.");
        app.emit("process-clipboard-done", "未识别为HTTP或SQL").ok();
        Err("未识别为HTTP或SQL".to_string())
    }
}

/// 处理HTTP内容
async fn handle_http_content(app: &AppHandle, content: &str) -> Result<String, String> {
    println!("[Clipboard] 判断为 HTTP 请求.");
    if content.contains("curl -X") {
        app.emit("process-clipboard-done", "HTTP请求已格式化，无需处理").ok();
        return Ok("HTTP请求已格式化，无需处理".to_string());
    }
    
    let (curl, title) = if content.contains("http.body:") {
        let parts: Vec<&str> = content.splitn(2, "http.body:").collect();
        let url = parts.get(0).map(|s| s.trim()).unwrap_or("");
        let json = parts.get(1).map(|s| s.trim()).unwrap_or("");
        let url_path = extract_url_path(url);
        let curl = format!("curl -X POST -H \"Accept-Language:zh-CN\" -H \"logLevel:debug\" -H \"Content-Type:application/json\" -d '{{}}' --url \"http://localhost:8080/{}\"", url_path);
        let curl = curl.replace("'{}'", &format!("'{}'", json.replace("'", "'\"'\"'")));
        (curl, format!("POST - {}", url_path))
    } else {
        let url = content.trim();
        let url_path = extract_url_path(url);
        let curl = format!("curl -X GET -H \"Accept-Language:zh-CN\" -H \"logLevel:debug\" --url \"http://localhost:8080/{}\"", url_path);
        (curl, format!("GET - {}", url_path))
    };
    
    if let Ok(mut clipboard) = Clipboard::new() {
        clipboard.set_text(&curl).ok();
    }
    
    let record = ParseRecord {
        id: format!("{}-{}", chrono::Utc::now().timestamp_millis(), rand::random::<u32>()),
        timestamp: chrono::Utc::now().timestamp_millis(),
        request_type: if content.contains("http.body:") { "POST".to_string() } else { "GET".to_string() },
        url: content.to_string(),
        json_data: None,
        curl_command: curl.clone(),
        title: Some(title.clone()),
    };
    
    crate::desktop::store::save_parse_record_internal(app.clone(), record).await?;
    
    app.emit("process-clipboard-done", "HTTP请求已格式化并写入剪贴板").ok();
    Ok("HTTP请求已格式化并写入剪贴板".to_string())
}

/// 处理SQL内容
async fn handle_sql_content(app: &AppHandle, content: &str) -> Result<String, String> {
    println!("[Clipboard] 判断为 SQL 请求.");
    if content.contains('\n') && !content.contains("db.sql.parameters:") {
        println!("[Clipboard] SQL 已被格式化, 跳过处理.");
        app.emit("process-clipboard-done", "SQL已格式化，无需处理").ok();
        return Ok("SQL已格式化，无需处理".to_string());
    }
    
    println!("[Clipboard] 准备格式化 SQL...");
    let formatted = format_sql_string(content);
    
    if let Ok(mut clipboard) = Clipboard::new() {
        clipboard.set_text(&formatted).ok();
    }
    
    crate::desktop::store::save_sql_history_internal(app.clone(), content, &formatted).await?;
    
    println!("[Clipboard] SQL 格式化完成并已写入剪贴板.");
    app.emit("process-clipboard-done", "SQL已格式化并写入剪贴板").ok();
    Ok("SQL已格式化并写入剪贴板".to_string())
}

/// 提取URL路径
fn extract_url_path(url: &str) -> String {
    if let Some(idx) = url.find("//") {
        let rest = &url[idx+2..];
        if let Some(idx2) = rest.find('/') {
            return rest[idx2+1..].to_string();
        }
    }
    url.trim_start_matches("/").to_string()
}

/// 格式化SQL字符串
fn format_sql_string(sql_with_params: &str) -> String {
    let mut sql_statement = sql_with_params;
    let mut params: Vec<String> = Vec::new();

    if let Some(index) = sql_with_params.find("db.sql.parameters:") {
        let (stmt, params_str_with_prefix) = sql_with_params.split_at(index);
        sql_statement = stmt.trim();
        let params_part = params_str_with_prefix.replace("db.sql.parameters:", "");
        let params_str = params_part.trim();
        
        if params_str.starts_with('[') && params_str.ends_with(']') {
            let inner_params = &params_str[1..params_str.len()-1];
            params = inner_params.split(',')
                .map(|p| p.trim().to_string())
                .collect();
        }
    }

    let mut final_sql = sql_statement.to_string();
    
    for param in params {
        let is_numeric = param.parse::<f64>().is_ok();
        let is_string_literal = (param.starts_with('\'') && param.ends_with('\'')) || (param.starts_with('"') && param.ends_with('"'));
        
        let replacement = if is_numeric || is_string_literal {
            param
        } else {
            format!("'{}'", param)
        };
        final_sql = final_sql.replacen('?', &replacement, 1);
    }

    final_sql = final_sql.replace("SELECT", "\nSELECT")
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
        
    if final_sql.starts_with('\n') {
        final_sql = final_sql[1..].to_string();
    }
    
    final_sql.trim().to_string()
} 