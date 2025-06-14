use arboard::Clipboard;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use tauri_plugin_store::StoreExt;

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

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_clipboard() -> Result<String, String> {
    match Clipboard::new() {
        Ok(mut clipboard) => {
            match clipboard.get_text() {
                Ok(text) => Ok(text),
                Err(e) => Err(format!("Failed to get clipboard text: {}", e))
            }
        },
        Err(e) => Err(format!("Failed to create clipboard: {}", e))
    }
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
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            greet, 
            get_clipboard, 
            save_parse_record, 
            get_parse_records, 
            delete_parse_record, 
            clear_parse_records
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
