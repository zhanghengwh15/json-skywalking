use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SqlHistoryItem {
    pub sql: String,
    pub formatted_sql: String,
    pub timestamp: i64,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonHistoryItem {
    pub data: serde_json::Value,
    pub timestamp: i64,
    pub hash: String,
}

/// 保存解析记录（内部函数）
pub async fn save_parse_record_internal(
    app: tauri::AppHandle,
    record: ParseRecord,
) -> Result<(), String> {
    let store = app.store("parse_history.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;
    
    let mut records: Vec<ParseRecord> = store
        .get("records")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    
    records.insert(0, record);
    
    if records.len() > 100 {
        records.truncate(100);
    }
    
    store.set("records".to_string(), serde_json::to_value(&records).unwrap());
    store.save()
        .map_err(|e| format!("Failed to save store: {}", e))?;
    
    Ok(())
}

/// 保存解析记录（对外API）
#[tauri::command]
pub async fn save_parse_record(
    app: tauri::AppHandle,
    record: ParseRecord,
) -> Result<(), String> {
    save_parse_record_internal(app, record).await
}

/// 获取解析记录
#[tauri::command]
pub async fn get_parse_records(app: tauri::AppHandle) -> Result<Vec<ParseRecord>, String> {
    let store = app.store("parse_history.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;
    
    let records: Vec<ParseRecord> = store
        .get("records")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    
    Ok(records)
}

/// 删除解析记录
#[tauri::command]
pub async fn delete_parse_record(
    app: tauri::AppHandle,
    record_id: String,
) -> Result<(), String> {
    let store = app.store("parse_history.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;
    
    let mut records: Vec<ParseRecord> = store
        .get("records")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    
    records.retain(|r| r.id != record_id);
    
    store.set("records".to_string(), serde_json::to_value(&records).unwrap());
    store.save()
        .map_err(|e| format!("Failed to save store: {}", e))?;
    
    Ok(())
}

/// 清空解析记录
#[tauri::command]
pub async fn clear_parse_records(app: tauri::AppHandle) -> Result<(), String> {
    let store = app.store("parse_history.json")
        .map_err(|e| format!("Failed to access store: {}", e))?;
    
    store.set("records".to_string(), serde_json::to_value(Vec::<ParseRecord>::new()).unwrap());
    store.save()
        .map_err(|e| format!("Failed to save store: {}", e))?;
    
    Ok(())
}

/// 生成哈希值
fn generate_hash(sql: &str) -> String {
    let mut hash = 0i64;
    for b in sql.bytes() {
        hash = ((hash << 5) - hash) + b as i64;
        hash &= 0xFFFFFFFF;
    }
    format!("{:x}", hash.abs())
}

/// 保存SQL历史记录（内部函数）
pub async fn save_sql_history_internal(
    app: tauri::AppHandle,
    sql: &str,
    formatted_sql: &str,
) -> Result<(), String> {
    println!("[SQL History] 开始保存历史记录...");
    let app_dir = match app.path().app_data_dir() {
        Ok(dir) => {
            println!("[SQL History] 应用数据目录: {:?}", dir);
            dir
        },
        Err(e) => {
            eprintln!("[SQL History] 错误: 获取应用数据目录失败: {}", e);
            return Err(e.to_string());
        }
    };

    if let Err(e) = std::fs::create_dir_all(&app_dir) {
        eprintln!("[SQL History] 错误: 创建目录失败: {}", e);
    }

    let history_file = app_dir.join("sql_history.json");
    println!("[SQL History] 历史文件路径: {:?}", history_file);

    let mut items: Vec<SqlHistoryItem> = if history_file.exists() {
        println!("[SQL History] 历史文件存在, 正在读取...");
        let json_str = std::fs::read_to_string(&history_file).unwrap_or_default();
        serde_json::from_str(&json_str).unwrap_or_else(|e| {
            eprintln!("[SQL History] 警告: 解析历史文件失败: {}, 将创建新历史.", e);
            Vec::new()
        })
    } else {
        println!("[SQL History] 历史文件不存在, 创建新列表.");
        Vec::new()
    };

    let hash = generate_hash(sql);
    if !items.iter().any(|item| item.hash == hash) {
        println!("[SQL History] 新的 SQL, 准备插入记录.");
        items.insert(0, SqlHistoryItem {
            sql: sql.to_string(),
            formatted_sql: formatted_sql.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            hash,
        });
        if items.len() > 20 { items.truncate(20); }
        
        match serde_json::to_string_pretty(&items) {
            Ok(json_str) => {
                if let Err(e) = std::fs::write(&history_file, json_str) {
                    eprintln!("[SQL History] 错误: 写入历史文件失败: {}", e);
                    return Err(e.to_string());
                } else {
                    println!("[SQL History] 历史记录写入成功.");
                }
            },
            Err(e) => {
                eprintln!("[SQL History] 错误: 序列化历史记录失败: {}", e);
                return Err(e.to_string());
            }
        }
    } else {
        println!("[SQL History] SQL 已存在于历史中, 跳过保存.");
    }

    Ok(())
}

/// 加载SQL历史记录
#[tauri::command]
pub fn load_sql_history(app_handle: tauri::AppHandle) -> Result<Vec<SqlHistoryItem>, String> {
    let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let history_file = app_dir.join("sql_history.json");

    if !history_file.exists() {
        return Ok(Vec::new());
    }

    let json_str = std::fs::read_to_string(history_file).map_err(|e| e.to_string())?;
    
    serde_json::from_str(&json_str).map_err(|e| {
        eprintln!("[SQL History] 读取错误: {}", e);
        e.to_string()
    })
}

/// 保存SQL历史记录（对外API）
#[tauri::command]
pub async fn save_sql_history(app: tauri::AppHandle, history: Vec<SqlHistoryItem>) -> Result<(), String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let history_file = app_dir.join("sql_history.json");

    if let Err(e) = std::fs::create_dir_all(&app_dir) {
        eprintln!("[SQL History] 错误: 创建目录失败: {}", e);
        return Err(e.to_string());
    }

    let json_str = serde_json::to_string_pretty(&history).map_err(|e| e.to_string())?;
    
    if let Err(e) = std::fs::write(&history_file, json_str) {
        eprintln!("[SQL History] 错误: 写入历史文件失败: {}", e);
        return Err(e.to_string());
    } else {
        println!("[SQL History] 历史记录写入成功.");
        Ok(())
    }
}

/// 加载JSON历史记录
#[tauri::command]
pub fn load_json_history(app_handle: tauri::AppHandle) -> Result<Vec<JsonHistoryItem>, String> {
    let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let history_file = app_dir.join("json_parser_history.json");

    if !history_file.exists() {
        return Ok(Vec::new());
    }

    let json_str = std::fs::read_to_string(history_file).map_err(|e| e.to_string())?;
    
    if json_str.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str(&json_str).map_err(|e| e.to_string())
}

/// 保存JSON历史记录
#[tauri::command]
pub async fn save_json_history(
    app: tauri::AppHandle,
    history: Vec<JsonHistoryItem>,
) -> Result<(), String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    }
    let history_file = app_dir.join("json_parser_history.json");

    let json_str = serde_json::to_string_pretty(&history).map_err(|e| e.to_string())?;

    std::fs::write(history_file, json_str).map_err(|e| e.to_string())
} 