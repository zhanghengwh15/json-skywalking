use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tower_http::limit::RequestBodyLimitLayer;
use serde::Serialize;
use std::io;
use std::net::SocketAddr;
use tauri::Emitter;
use tokio::net::TcpListener;
use tokio::task;

use super::db::{Db, PushPayload};

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

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/push", post(push_handler))
        .route("/domains", get(list_domains_handler))
        .route("/domains/:domain", get(get_domain_handler))
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state)
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

pub async fn serve(addr: SocketAddr, state: AppState) -> Result<(), io::Error> {
    let listener = TcpListener::bind(addr).await?;
    let router = create_router(state);
    axum::serve(listener, router).await
}
