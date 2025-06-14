use arboard::Clipboard;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_clipboard])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
