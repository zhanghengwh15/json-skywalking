use tauri::State;

use super::db::{Db, DomainSnapshot};

#[tauri::command]
pub async fn cookie_bridge_list_domains(state: State<'_, Db>) -> Result<Vec<String>, String> {
    let db = state.inner().clone();
    tokio::task::spawn_blocking(move || db.list_domains())
        .await
        .map_err(|e| format!("task error: {}", e))?
        .map_err(|e| format!("db error: {}", e))
}

#[tauri::command]
pub async fn cookie_bridge_get_domain(
    domain: String,
    state: State<'_, Db>,
) -> Result<DomainSnapshot, String> {
    let db = state.inner().clone();
    tokio::task::spawn_blocking(move || db.get_domain(&domain))
        .await
        .map_err(|e| format!("task error: {}", e))?
        .map_err(|e| format!("db error: {}", e))
}
