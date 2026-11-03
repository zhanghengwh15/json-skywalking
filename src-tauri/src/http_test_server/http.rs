use super::{LogStore, LogSummary, RequestLog, ServerConfig, MAX_LOG_BODY, MAX_REQUEST_BODY};
use axum::{
    body::to_bytes,
    extract::{ConnectInfo, Request, State},
    http::{header, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    Router,
};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::time::{timeout, Duration};

#[derive(Clone)]
struct HttpState {
    config: ServerConfig,
    logs: Arc<Mutex<LogStore>>,
}

pub fn create_router(config: ServerConfig, logs: Arc<Mutex<LogStore>>) -> Router {
    Router::new()
        .fallback(receive)
        .with_state(HttpState { config, logs })
}

async fn receive(
    State(state): State<HttpState>,
    ConnectInfo(remote): ConnectInfo<SocketAddr>,
    request: Request,
) -> Response {
    let started = Instant::now();
    let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let (parts, body) = request.into_parts();
    let endpoint = state.config.endpoints.iter().find(|endpoint| {
        endpoint.path == parts.uri.path()
            && (endpoint.method == "ANY" || endpoint.method == parts.method.as_str())
    });
    let headers = parts
        .headers
        .iter()
        .map(|(name, value)| {
            let hidden = matches!(
                name.as_str(),
                "authorization" | "proxy-authorization" | "cookie" | "x-api-key"
            ) || name.as_str().contains("token")
                || name.as_str().contains("secret");
            (
                name.to_string(),
                if hidden {
                    "[已隐藏]".into()
                } else {
                    String::from_utf8_lossy(value.as_bytes()).into_owned()
                },
            )
        })
        .collect();

    let (body_text, truncated, body_error) =
        match timeout(Duration::from_secs(15), to_bytes(body, MAX_REQUEST_BODY)).await {
            Ok(Ok(bytes)) => (
                String::from_utf8_lossy(&bytes[..bytes.len().min(MAX_LOG_BODY)]).into_owned(),
                bytes.len() > MAX_LOG_BODY,
                None,
            ),
            Ok(Err(error)) => {
                let too_large = error
                    .source()
                    .is_some_and(|source| source.is::<http_body_util::LengthLimitError>());
                (
                    String::new(),
                    true,
                    Some(if too_large {
                        (
                            StatusCode::PAYLOAD_TOO_LARGE,
                            r#"{"error":"请求体不能超过 1 MiB"}"#,
                        )
                    } else {
                        (StatusCode::BAD_REQUEST, r#"{"error":"请求体读取失败"}"#)
                    }),
                )
            }
            Err(_) => (
                String::new(),
                true,
                Some((StatusCode::REQUEST_TIMEOUT, r#"{"error":"请求体读取超时"}"#)),
            ),
        };

    let (status, mut response_body) = if let Some((status, message)) = body_error {
        (status, message.to_owned())
    } else if parts.method == Method::OPTIONS {
        (StatusCode::NO_CONTENT, String::new())
    } else if let Some(endpoint) = endpoint {
        (
            StatusCode::from_u16(endpoint.status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            endpoint.response_body.clone(),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            r#"{"error":"未配置该路径或请求方法"}"#.into(),
        )
    };
    if parts.method == Method::HEAD
        || status == StatusCode::NO_CONTENT
        || status == StatusCode::RESET_CONTENT
        || status == StatusCode::NOT_MODIFIED
    {
        response_body.clear();
    }
    let entry = RequestLog {
        summary: LogSummary {
            id: 0,
            timestamp,
            port: state.config.port,
            method: parts.method.to_string(),
            url: format!("http://127.0.0.1:{}{}", state.config.port, parts.uri),
            remote_address: remote.to_string(),
            status_code: status.as_u16(),
            duration_ms: started.elapsed().as_millis() as u64,
            matched_path: endpoint.map(|endpoint| endpoint.path.clone()),
        },
        headers,
        body: body_text,
        body_truncated: truncated,
        response_body: response_body.clone(),
    };
    if let Ok(mut logs) = state.logs.lock() {
        logs.push(entry);
    } else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "请求日志不可用").into_response();
    }
    let mut response = (
        status,
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        response_body,
    )
        .into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS"),
    );
    if let Some(requested) = parts.headers.get(header::ACCESS_CONTROL_REQUEST_HEADERS) {
        headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, requested.clone());
    }
    response
}
