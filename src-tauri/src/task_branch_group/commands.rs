use tauri::State;

use crate::cookie_bridge::db::Db;
use crate::task_branch_group::db::{CreateTaskBranchGroup, TaskBranchGroup, UpdateTaskBranchGroup};

#[tauri::command]
pub async fn task_branch_group_create(
    item: CreateTaskBranchGroup,
    state: State<'_, Db>,
) -> Result<i64, String> {
    let db = state.inner().clone();
    tokio::task::spawn_blocking(move || db.task_branch_group_create(&item))
        .await
        .map_err(|e| format!("task error: {}", e))?
        .map_err(|e| format!("db error: {}", e))
}

#[tauri::command]
pub async fn task_branch_group_list(
    keyword: Option<String>,
    state: State<'_, Db>,
) -> Result<Vec<TaskBranchGroup>, String> {
    let db = state.inner().clone();
    tokio::task::spawn_blocking(move || db.task_branch_group_list(keyword.as_deref(), None, None))
        .await
        .map_err(|e| format!("task error: {}", e))?
        .map_err(|e| format!("db error: {}", e))
}

#[tauri::command]
pub async fn task_branch_group_get(
    id: i64,
    state: State<'_, Db>,
) -> Result<Option<TaskBranchGroup>, String> {
    let db = state.inner().clone();
    tokio::task::spawn_blocking(move || db.task_branch_group_get(id))
        .await
        .map_err(|e| format!("task error: {}", e))?
        .map_err(|e| format!("db error: {}", e))
}

#[tauri::command]
pub async fn task_branch_group_update(
    id: i64,
    item: UpdateTaskBranchGroup,
    state: State<'_, Db>,
) -> Result<bool, String> {
    let db = state.inner().clone();
    tokio::task::spawn_blocking(move || db.task_branch_group_update(id, &item))
        .await
        .map_err(|e| format!("task error: {}", e))?
        .map_err(|e| format!("db error: {}", e))
}

#[tauri::command]
pub async fn task_branch_group_delete(id: i64, state: State<'_, Db>) -> Result<bool, String> {
    let db = state.inner().clone();
    tokio::task::spawn_blocking(move || db.task_branch_group_delete(id))
        .await
        .map_err(|e| format!("task error: {}", e))?
        .map_err(|e| format!("db error: {}", e))
}
