use tauri::State;

use super::db::{Db, DomainSnapshot, Domain, CreateDomain, UpdateDomain};
use super::{is_debug_mode, set_debug_mode};

#[tauri::command]
pub async fn cookie_bridge_list_domains(state: State<'_, Db>) -> Result<Vec<Domain>, String> {
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

#[tauri::command]
pub async fn cookie_bridge_set_debug_mode(enabled: bool) {
    set_debug_mode(enabled);
}

#[tauri::command]
pub async fn cookie_bridge_get_debug_mode() -> bool {
    is_debug_mode()
}

#[tauri::command]
pub async fn cookie_bridge_delete_domain(
    domain: String,
    state: State<'_, Db>,
) -> Result<(usize, usize), String> {
    let db = state.inner().clone();
    tokio::task::spawn_blocking(move || db.delete_domain(&domain))
        .await
        .map_err(|e| format!("task error: {}", e))?
        .map_err(|e| format!("db error: {}", e))
}

// Domain CRUD commands

#[tauri::command]
pub async fn domain_list(state: State<'_, Db>) -> Result<Vec<Domain>, String> {
    let db = state.inner().clone();
    tokio::task::spawn_blocking(move || db.domain_list())
        .await
        .map_err(|e| format!("task error: {}", e))?
        .map_err(|e| format!("db error: {}", e))
}

#[tauri::command]
pub async fn domain_create(
    payload: CreateDomain,
    state: State<'_, Db>,
) -> Result<Domain, String> {
    let db = state.inner().clone();
    let id = tokio::task::spawn_blocking(move || db.domain_create(&payload))
        .await
        .map_err(|e| format!("task error: {}", e))?
        .map_err(|e| format!("db error: {}", e))?;

    db.domain_get(id)
        .map(|opt| opt.ok_or_else(|| "created item not found".to_string()))
        .map_err(|e| format!("db error: {}", e))?
}

#[tauri::command]
pub async fn domain_get(id: i64, state: State<'_, Db>) -> Result<Domain, String> {
    let db = state.inner().clone();
    tokio::task::spawn_blocking(move || db.domain_get(id))
        .await
        .map_err(|e| format!("task error: {}", e))?
        .map_err(|e| format!("db error: {}", e))?
        .ok_or_else(|| "not found".to_string())
}

#[tauri::command]
pub async fn domain_update(
    id: i64,
    payload: UpdateDomain,
    state: State<'_, Db>,
) -> Result<Domain, String> {
    let db = state.inner().clone();
    let updated = tokio::task::spawn_blocking(move || db.domain_update(id, &payload))
        .await
        .map_err(|e| format!("task error: {}", e))?
        .map_err(|e| format!("db error: {}", e))?;

    if !updated {
        return Err("not found or no changes".to_string());
    }

    db.domain_get(id)
        .map(|opt| opt.ok_or_else(|| "updated item not found".to_string()))
        .map_err(|e| format!("db error: {}", e))?
}

#[tauri::command]
pub async fn domain_delete(id: i64, state: State<'_, Db>) -> Result<bool, String> {
    let db = state.inner().clone();
    tokio::task::spawn_blocking(move || db.domain_delete(id))
        .await
        .map_err(|e| format!("task error: {}", e))?
        .map_err(|e| format!("db error: {}", e))
}

#[tauri::command]
pub async fn domain_match_url(
    url: String,
    state: State<'_, Db>,
) -> Result<super::http::MatchResult, String> {
    let db = state.inner().clone();
    let result = tokio::task::spawn_blocking(move || db.domain_match_url(&url))
        .await
        .map_err(|e| format!("task error: {}", e))?
        .map_err(|e| format!("db error: {}", e))?;

    match result {
        Some((domain, cookies, local_storage)) => Ok(super::http::MatchResult {
            domain: Some(domain),
            cookies,
            local_storage,
        }),
        None => Ok(super::http::MatchResult {
            domain: None,
            cookies: vec![],
            local_storage: vec![],
        }),
    }
}
