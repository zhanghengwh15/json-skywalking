use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use tower_http::limit::RequestBodyLimitLayer;
use serde::{Deserialize, Serialize};
use std::io;
use std::net::SocketAddr;
use tauri::Emitter;
use tokio::net::TcpListener;
use tokio::task;

use super::db::{Db, PushPayload, CreateDomain, UpdateDomain, Domain};

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub app_handle: tauri::AppHandle,
}

#[derive(Debug, Serialize)]
pub struct PushResponse {
    pub ok: bool,
    pub cookies: usize,
    pub local_storage: usize,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

fn success_response<T: Serialize>(data: T) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            data: Some(data),
            message: None,
        }),
    )
        .into_response()
}

fn error_response(status: StatusCode, message: &str) -> impl IntoResponse {
    (
        status,
        Json(ApiResponse::<serde_json::Value> {
            success: false,
            data: None,
            message: Some(message.to_string()),
        }),
    )
        .into_response()
}

pub fn create_router(state: AppState) -> Router {
    let tbg_router = crate::task_branch_group::http::create_router();
    let domain_router = create_domain_router();

    Router::new()
        .route("/health", get(health_handler))
        .route("/push", post(push_handler))
        .route("/domains", get(list_domains_handler))
        .route("/domains/:domain", get(get_domain_handler))
        .nest("/api/domains", domain_router)
        .nest("/api/task-branch-groups", tbg_router)
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state)
}

fn create_domain_router() -> Router<AppState> {
    Router::new()
        .route("/", get(domain_list_handler))
        .route("/", post(domain_create_handler))
        .route("/:id", get(domain_get_handler))
        .route("/:id", put(domain_update_handler))
        .route("/:id", delete(domain_delete_handler))
        .route("/match", get(domain_match_handler))
}

async fn health_handler() -> impl IntoResponse {
    "ok"
}

async fn push_handler(
    State(state): State<AppState>,
    Json(payload): Json<PushPayload>,
) -> impl IntoResponse {
    if payload.domain.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "domain is required".to_string(),
            }),
        )
            .into_response();
    }

    let db = state.db.clone();
    let domain = payload.domain.clone();
    let result = task::spawn_blocking(move || db.push(&payload)).await;

    match result {
        Ok(Ok((cookies, local_storage))) => {
            let _ = state
                .app_handle
                .emit("cookie-bridge:updated", &domain);
            (
                StatusCode::OK,
                Json(PushResponse {
                    ok: true,
                    cookies,
                    local_storage,
                }),
            )
                .into_response()
        }
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("db error: {}", e),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("task error: {}", e),
            }),
        )
            .into_response(),
    }
}

async fn list_domains_handler(State(state): State<AppState>) -> impl IntoResponse {
    let db = state.db.clone();
    match task::spawn_blocking(move || db.list_domains()).await {
        Ok(Ok(domains)) => (StatusCode::OK, Json(domains)).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("db error: {}", e),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("task error: {}", e),
            }),
        )
            .into_response(),
    }
}

async fn get_domain_handler(
    State(state): State<AppState>,
    Path(domain): Path<String>,
) -> impl IntoResponse {
    let db = state.db.clone();
    match task::spawn_blocking(move || db.get_domain(&domain)).await {
        Ok(Ok(snapshot)) => (StatusCode::OK, Json(snapshot)).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("db error: {}", e),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("task error: {}", e),
            }),
        )
            .into_response(),
    }
}

// Domain CRUD handlers

async fn domain_list_handler(State(state): State<AppState>) -> impl IntoResponse {
    let db = state.db.clone();
    match task::spawn_blocking(move || db.domain_list()).await {
        Ok(Ok(items)) => success_response(items).into_response(),
        Ok(Err(e)) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {}", e)).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("task error: {}", e)).into_response(),
    }
}

async fn domain_create_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateDomain>,
) -> impl IntoResponse {
    if payload.domain_name.is_empty() {
        return error_response(StatusCode::BAD_REQUEST, "domain_name is required").into_response();
    }
    let db = state.db.clone();
    match task::spawn_blocking(move || db.domain_create(&payload)).await {
        Ok(Ok(id)) => {
            let db = state.db.clone();
            match db.domain_get(id) {
                Ok(Some(item)) => success_response(item).into_response(),
                Ok(None) => error_response(StatusCode::NOT_FOUND, "created item not found").into_response(),
                Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {}", e)).into_response(),
            }
        }
        Ok(Err(e)) => {
            let msg = e.to_string();
            if msg.contains("UNIQUE") {
                error_response(StatusCode::CONFLICT, "domain already exists").into_response()
            } else {
                error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {}", e)).into_response()
            }
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("task error: {}", e)).into_response(),
    }
}

async fn domain_get_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = state.db.clone();
    match task::spawn_blocking(move || db.domain_get(id)).await {
        Ok(Ok(Some(item))) => success_response(item).into_response(),
        Ok(Ok(None)) => error_response(StatusCode::NOT_FOUND, "not found").into_response(),
        Ok(Err(e)) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {}", e)).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("task error: {}", e)).into_response(),
    }
}

async fn domain_update_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateDomain>,
) -> impl IntoResponse {
    let db = state.db.clone();
    match task::spawn_blocking(move || db.domain_update(id, &payload)).await {
        Ok(Ok(true)) => {
            let db = state.db.clone();
            match db.domain_get(id) {
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

async fn domain_delete_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let db = state.db.clone();
    match task::spawn_blocking(move || db.domain_delete(id)).await {
        Ok(Ok(true)) => success_response(serde_json::json!({"deleted": true})).into_response(),
        Ok(Ok(false)) => error_response(StatusCode::NOT_FOUND, "not found").into_response(),
        Ok(Err(e)) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {}", e)).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("task error: {}", e)).into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct MatchQuery {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct MatchResult {
    pub domain: Option<Domain>,
    pub cookies: Vec<super::db::CookieItem>,
    pub local_storage: Vec<super::db::LocalStorageItem>,
}

async fn domain_match_handler(
    State(state): State<AppState>,
    Query(query): Query<MatchQuery>,
) -> impl IntoResponse {
    let db = state.db.clone();
    match task::spawn_blocking(move || db.domain_match_url(&query.url)).await {
        Ok(Ok(Some((domain, cookies, local_storage)))) => {
            success_response(MatchResult {
                domain: Some(domain),
                cookies,
                local_storage,
            }).into_response()
        }
        Ok(Ok(None)) => {
            success_response(MatchResult {
                domain: None,
                cookies: vec![],
                local_storage: vec![],
            }).into_response()
        }
        Ok(Err(e)) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("db error: {}", e)).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &format!("task error: {}", e)).into_response(),
    }
}

pub async fn serve(addr: SocketAddr, state: AppState) -> Result<(), io::Error> {
    let listener = TcpListener::bind(addr).await?;
    let router = create_router(state);
    axum::serve(listener, router).await
}
