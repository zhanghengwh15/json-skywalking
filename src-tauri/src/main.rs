// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use serde::{Deserialize, Serialize};
use tauri::Manager;
use arboard::Clipboard;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct HistoryItem {
    data: serde_json::Value,
    timestamp: i64,
    hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonHistory {
    items: Vec<HistoryItem>,
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

#[tauri::command]
fn save_json_history(app_handle: tauri::AppHandle, history: Vec<HistoryItem>) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    
    // 确保目录存在
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    
    let history_file = app_dir.join("json_history.json");
    let json_history = JsonHistory { items: history };
    
    let json_str = serde_json::to_string_pretty(&json_history)
        .map_err(|e| e.to_string())?;
    
    fs::write(history_file, json_str)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn load_json_history(app_handle: tauri::AppHandle) -> Result<Vec<HistoryItem>, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    
    let history_file = app_dir.join("json_history.json");
    
    if !history_file.exists() {
        return Ok(Vec::new());
    }
    
    let json_str = fs::read_to_string(history_file)
        .map_err(|e| e.to_string())?;
    
    let json_history: JsonHistory = serde_json::from_str(&json_str)
        .map_err(|e| e.to_string())?;
    
    Ok(json_history.items)
}

#[tauri::command]
fn get_clipboard(_app_handle: tauri::AppHandle) -> Result<String, String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.get_text().map_err(|e| e.to_string())
}

#[tauri::command]
fn save_sql_history(app_handle: tauri::AppHandle, history: Vec<SqlHistoryItem>) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    
    // 确保目录存在
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    
    let history_file = app_dir.join("sql_history.json");
    let json_history = SqlHistory { items: history };
    
    let json_str = serde_json::to_string_pretty(&json_history)
        .map_err(|e| e.to_string())?;
    
    fs::write(history_file, json_str)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn load_sql_history(app_handle: tauri::AppHandle) -> Result<Vec<SqlHistoryItem>, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    
    let history_file = app_dir.join("sql_history.json");
    
    if !history_file.exists() {
        return Ok(Vec::new());
    }
    
    let json_str = fs::read_to_string(history_file)
        .map_err(|e| e.to_string())?;
    
    let json_history: SqlHistory = serde_json::from_str(&json_str)
        .map_err(|e| e.to_string())?;
    
    Ok(json_history.items)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            get_clipboard,
            save_json_history,
            load_json_history,
            save_sql_history,
            load_sql_history
        ])
        .run(tauri::generate_context!())
        .expect("运行应用程序时出错");
}
// 