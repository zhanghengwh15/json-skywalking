use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::task;

use crate::cookie_bridge::http::AppState;
use crate::task_branch_group::db::{CreateTaskBranchGroup, UpdateTaskBranchGroup};

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: Option<T>,
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub message: String,
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_handler))
        .route("/", post(create_handler))
        .route("/:id", get(get_handler))
        .route("/:id", put(update_handler))
        .route("/:id", delete(delete_handler))
}

fn success_response<T: Serialize>(data: T) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse {
            data: Some(data),
            success: true,
            message: None,
        }),
    )
        .into_response()
}

fn error_response(status: StatusCode, message: &str) -> impl IntoResponse {
    (
        status,
        Json(ErrorResponse {
            success: false,
            message: message.to_string(),
        }),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub keyword: Option<String>,
    pub task_id: Option<String>,
    pub branch_name: Option<String>,
}

async fn list_handler(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> impl IntoResponse {
    let db = state.db.clone();
    match task::spawn_blocking(move || {
        db.task_branch_group_list(
            query.keyword.as_deref(),
            query.task_id.as_deref(),
            query.branch_name.as_deref(),
        )
    })
    .await
    {
        Ok(Ok(items)) => success_response(items).into_response(),
        Ok(Err(e)) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {}", e)).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("task error: {}", e)).into_response(),
    }
}

async fn create_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskBranchGroup>,
) -> impl IntoResponse {
    let db = state.db.clone();
    match task::spawn_blocking(move || db.task_branch_group_create(&payload)).await {
        Ok(Ok(id)) => {
            let db = state.db.clone();
            match db.task_branch_group_get(id) {
                Ok(Some(item)) => success_response(item).into_response(),
                Ok(None) => error_response(StatusCode::NOT_FOUND, "created item not found").into_response(),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {}", e)).into_response(),
            }
        }
        Ok(Err(e)) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {}", e)).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("task error: {}", e)).into_response(),
    }
}

async fn get_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = state.db.clone();
    match task::spawn_blocking(move || db.task_branch_group_get(id)).await {
        Ok(Ok(Some(item))) => success_response(item).into_response(),
        Ok(Ok(None)) => error_response(StatusCode::NOT_FOUND, "not found").into_response(),
        Ok(Err(e)) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {}", e)).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("task error: {}", e)).into_response(),
    }
}

async fn update_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTaskBranchGroup>,
) -> impl IntoResponse {
    let db = state.db.clone();
    match task::spawn_blocking(move || db.task_branch_group_update(id, &payload)).await {
        Ok(Ok(true)) => {
            let db = state.db.clone();
            match db.task_branch_group_get(id) {
                Ok(Some(item)) => success_response(item).into_response(),
                Ok(None) => error_response(StatusCode::NOT_FOUND, "updated item not found").into_response(),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {}", e)).into_response(),
            }
        }
        Ok(Ok(false)) => error_response(StatusCode::NOT_FOUND, "not found or no changes").into_response(),
        Ok(Err(e)) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {}", e)).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("task error: {}", e)).into_response(),
    }
}

async fn delete_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = state.db.clone();
    match task::spawn_blocking(move || db.task_branch_group_delete(id)).await {
        Ok(Ok(true)) => success_response(HashMap::from([("deleted", true)])).into_response(),
        Ok(Ok(false)) => error_response(StatusCode::NOT_FOUND, "not found").into_response(),
        Ok(Err(e)) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {}", e)).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("task error: {}", e)).into_response(),
    }
}
