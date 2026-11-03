pub mod commands;
mod http;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex as AsyncMutex};
use tokio::task::JoinHandle;
use tokio::time::{timeout, Duration};

pub const MAX_LOGS: usize = 200;
pub const MAX_LOG_BODY: usize = 64 * 1024;
pub const MAX_REQUEST_BODY: usize = 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub response_body: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub port: u16,
    pub endpoints: Vec<Endpoint>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8766,
            endpoints: vec![Endpoint {
                method: "ANY".into(),
                path: "/test".into(),
                status_code: 200,
                response_body: "{\n  \"code\": 0,\n  \"message\": \"ok\",\n  \"data\": null\n}"
                    .into(),
            }],
        }
    }
}

impl ServerConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.port < 1024 || self.port == 8765 {
            return Err("端口需为 1024–65535，且不能使用 Cookie Bridge 的 8765".into());
        }
        if self.endpoints.is_empty() || self.endpoints.len() > 50 {
            return Err("请配置 1–50 个接口".into());
        }
        for (index, endpoint) in self.endpoints.iter().enumerate() {
            if !["ANY", "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD"]
                .contains(&endpoint.method.as_str())
            {
                return Err(format!("接口 {} 的请求方法不支持", index + 1));
            }
            let valid_path = endpoint.path.starts_with('/')
                && !endpoint.path.starts_with("//")
                && endpoint.path.len() <= 2048
                && !endpoint.path.contains(['?', '#'])
                && endpoint.path.bytes().all(|byte| byte.is_ascii_graphic())
                && endpoint.path.parse::<axum::http::Uri>().is_ok();
            if !valid_path {
                return Err(format!(
                    "接口 {} 需填写以 / 开头的 URL 路径，不含查询参数、片段或空格；中文请 URL 编码",
                    index + 1
                ));
            }
            if !(200..=599).contains(&endpoint.status_code) {
                return Err(format!("接口 {} 状态码需为 200–599", index + 1));
            }
            if endpoint.response_body.len() > MAX_LOG_BODY {
                return Err(format!("接口 {} 固定响应不能超过 64 KiB", index + 1));
            }
            serde_json::from_str::<serde_json::Value>(&endpoint.response_body)
                .map_err(|error| format!("接口 {} 固定响应不是有效 JSON：{error}", index + 1))?;
            if self.endpoints[..index].iter().any(|previous| {
                previous.path == endpoint.path
                    && (previous.method == endpoint.method
                        || previous.method == "ANY"
                        || endpoint.method == "ANY")
            }) {
                return Err(format!("接口 {} 的路径与方法重复或与 ANY 冲突", index + 1));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogSummary {
    pub id: u64,
    pub timestamp: String,
    pub port: u16,
    pub method: String,
    pub url: String,
    pub remote_address: String,
    pub status_code: u16,
    pub duration_ms: u64,
    pub matched_path: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestLog {
    #[serde(flatten)]
    pub summary: LogSummary,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub body_truncated: bool,
    pub response_body: String,
}

#[derive(Default)]
pub struct LogStore {
    next_id: u64,
    entries: VecDeque<RequestLog>,
}

impl LogStore {
    pub fn push(&mut self, mut entry: RequestLog) {
        self.next_id += 1;
        entry.summary.id = self.next_id;
        self.entries.push_front(entry);
        self.entries.truncate(MAX_LOGS);
    }
}

struct RunningServer {
    shutdown: oneshot::Sender<()>,
    task: JoinHandle<std::io::Result<()>>,
}

struct ServiceState {
    config: ServerConfig,
    running: Option<RunningServer>,
    last_error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerStatus {
    pub running: bool,
    pub config: ServerConfig,
    pub last_error: Option<String>,
}

pub struct TestServer {
    state: AsyncMutex<ServiceState>,
    pub logs: Arc<Mutex<LogStore>>,
    config_path: PathBuf,
}

impl TestServer {
    pub fn new(config_path: PathBuf) -> Self {
        let loaded = match std::fs::read(&config_path) {
            Ok(bytes) => serde_json::from_slice::<ServerConfig>(&bytes)
                .map_err(|error| format!("读取测试服务配置失败：{error}"))
                .and_then(|config| config.validate().map(|()| config)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(ServerConfig::default())
            }
            Err(error) => Err(format!("读取测试服务配置失败：{error}")),
        };
        let (config, last_error) = match loaded {
            Ok(config) => (config, None),
            Err(error) => (ServerConfig::default(), Some(error)),
        };
        Self {
            state: AsyncMutex::new(ServiceState {
                config,
                running: None,
                last_error,
            }),
            logs: Arc::new(Mutex::new(LogStore::default())),
            config_path,
        }
    }

    async fn reap(state: &mut ServiceState) {
        if state
            .running
            .as_ref()
            .is_some_and(|server| server.task.is_finished())
        {
            if let Some(server) = state.running.take() {
                state.last_error = Some(match server.task.await {
                    Ok(Err(error)) => format!("测试服务异常退出：{error}"),
                    Err(error) => format!("测试服务任务异常退出：{error}"),
                    Ok(Ok(())) => "测试服务已结束，请重新启动".into(),
                });
            }
        }
    }

    pub async fn status(&self) -> ServerStatus {
        let mut state = self.state.lock().await;
        Self::reap(&mut state).await;
        ServerStatus {
            running: state.running.is_some(),
            config: state.config.clone(),
            last_error: state.last_error.clone(),
        }
    }

    pub async fn start(&self, config: ServerConfig) -> Result<ServerStatus, String> {
        config.validate()?;
        let mut state = self.state.lock().await;
        Self::reap(&mut state).await;
        if state.running.is_some() {
            return Err("测试服务已在运行，请先停止再修改配置".into());
        }
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, config.port))
            .await
            .map_err(|error| {
                format!(
                    "无法绑定 127.0.0.1:{}，请检查端口占用：{error}",
                    config.port
                )
            })?;
        let serialized = serde_json::to_vec_pretty(&config).map_err(|error| error.to_string())?;
        let config_path = self.config_path.clone();
        tauri::async_runtime::spawn_blocking(move || std::fs::write(config_path, serialized))
            .await
            .map_err(|error| format!("保存配置任务失败：{error}"))?
            .map_err(|error| format!("保存配置失败：{error}"))?;

        let router = http::create_router(config.clone(), self.logs.clone());
        let (shutdown, receiver) = oneshot::channel();
        let task = tokio::spawn(async move {
            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .with_graceful_shutdown(async {
                let _ = receiver.await;
            })
            .await
        });
        state.config = config.clone();
        state.running = Some(RunningServer { shutdown, task });
        state.last_error = None;
        Ok(ServerStatus {
            running: true,
            config,
            last_error: None,
        })
    }

    pub async fn stop(&self) -> Result<ServerStatus, String> {
        let mut state = self.state.lock().await;
        if let Some(server) = state.running.take() {
            let _ = server.shutdown.send(());
            let mut task = server.task;
            match timeout(Duration::from_secs(2), &mut task).await {
                Ok(Ok(Ok(()))) => {}
                Ok(Ok(Err(error))) => state.last_error = Some(format!("服务退出错误：{error}")),
                Ok(Err(error)) => state.last_error = Some(format!("服务任务退出错误：{error}")),
                Err(_) => {
                    task.abort();
                    let _ = task.await;
                }
            }
        }
        Ok(ServerStatus {
            running: false,
            config: state.config.clone(),
            last_error: state.last_error.clone(),
        })
    }
}

pub fn init(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let directory = app.path().app_data_dir()?;
    std::fs::create_dir_all(&directory)?;
    app.manage(TestServer::new(directory.join("http-test-server.json")));
    Ok(())
}
