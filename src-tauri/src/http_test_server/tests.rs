use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

struct Fixture {
    server: TestServer,
    config: ServerConfig,
    directory: PathBuf,
}

impl Fixture {
    async fn new() -> Self {
        let reserved = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = reserved.local_addr().unwrap().port();
        drop(reserved);
        let directory =
            std::env::temp_dir().join(format!("dev-tools-http-test-{}", rand::random::<u64>()));
        std::fs::create_dir_all(&directory).unwrap();
        let server = TestServer::new(directory.join("config.json"));
        let mut config = ServerConfig::default();
        config.port = port;
        config.endpoints[0].method = "POST".into();
        config.endpoints[0].status_code = 201;
        config.endpoints[0].response_body = r#"{"accepted":true}"#.into();
        Self {
            server,
            config,
            directory,
        }
    }

    async fn request(&self, method: &str, path: &str, body: &[u8], extra_headers: &str) -> String {
        let mut connection = TcpStream::connect(("127.0.0.1", self.config.port))
            .await
            .unwrap();
        let headers = format!("{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: {}\r\n{extra_headers}\r\n", body.len());
        connection.write_all(headers.as_bytes()).await.unwrap();
        // A server can reject an oversized body before all bytes have been sent.
        let write = connection.write_all(body).await;
        if body.len() <= MAX_REQUEST_BODY {
            write.unwrap();
        }
        let mut response = Vec::new();
        timeout(
            Duration::from_secs(5),
            connection.read_to_end(&mut response),
        )
        .await
        .unwrap()
        .unwrap();
        String::from_utf8(response).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[tokio::test]
async fn receives_arbitrary_bodies_and_records_request_response() {
    let fixture = Fixture::new().await;
    fixture.server.start(fixture.config.clone()).await.unwrap();
    for body in [
        br#"{"anything":[1,true]}"#.as_slice(),
        b"name=example&count=2",
        b"not json at all",
        b"",
    ] {
        let response = fixture.request("POST", "/test?arbitrary=yes", body,
            "Authorization: Bearer fake-test-token\r\nCookie: fake=value\r\nContent-Type: text/plain\r\n").await;
        assert!(response.starts_with("HTTP/1.1 201"), "{response}");
        assert!(response.ends_with(r#"{"accepted":true}"#));
    }
    let logs = fixture.server.logs.lock().unwrap();
    assert_eq!(logs.entries.len(), 4);
    let last = &logs.entries[0];
    assert!(last.summary.url.ends_with("/test?arbitrary=yes"));
    assert_eq!(last.summary.matched_path.as_deref(), Some("/test"));
    assert_eq!(last.summary.status_code, 201);
    assert_eq!(last.response_body, r#"{"accepted":true}"#);
    assert!(last
        .headers
        .iter()
        .any(|(key, value)| key == "authorization" && value == "[已隐藏]"));
    assert!(last
        .headers
        .iter()
        .any(|(key, value)| key == "cookie" && value == "[已隐藏]"));
    assert_eq!(logs.entries[1].body, "not json at all");
    drop(logs);
    fixture.server.stop().await.unwrap();
}

#[tokio::test]
async fn handles_missing_paths_methods_preflight_head_and_bodyless_statuses() {
    let mut fixture = Fixture::new().await;
    fixture.config.endpoints.push(Endpoint {
        method: "ANY".into(),
        path: "/any".into(),
        status_code: 200,
        response_body: "{}".into(),
    });
    fixture.config.endpoints.push(Endpoint {
        method: "GET".into(),
        path: "/empty".into(),
        status_code: 204,
        response_body: "{}".into(),
    });
    fixture.server.start(fixture.config.clone()).await.unwrap();
    assert!(fixture
        .request("GET", "/test", b"", "")
        .await
        .starts_with("HTTP/1.1 404"));
    assert!(fixture
        .request("POST", "/missing", b"", "")
        .await
        .starts_with("HTTP/1.1 404"));
    assert!(fixture
        .request("CUSTOM", "/any", b"hello", "")
        .await
        .starts_with("HTTP/1.1 200"));
    let preflight = fixture.request("OPTIONS", "/test", b"", "Origin: http://example.test\r\nAccess-Control-Request-Method: POST\r\nAccess-Control-Request-Headers: content-type,x-demo\r\n").await;
    assert!(preflight.starts_with("HTTP/1.1 204"));
    assert!(preflight.contains("access-control-allow-origin: *"));
    assert!(preflight.contains("access-control-allow-headers: content-type,x-demo"));
    for (method, path, status) in [("HEAD", "/any", 200), ("GET", "/empty", 204)] {
        let response = fixture.request(method, path, b"", "").await;
        assert!(response.starts_with(&format!("HTTP/1.1 {status}")));
        assert!(response.ends_with("\r\n\r\n"));
    }
    assert_eq!(fixture.server.logs.lock().unwrap().entries.len(), 6);
    fixture.server.stop().await.unwrap();
}

#[tokio::test]
async fn limits_body_and_log_retention() {
    let fixture = Fixture::new().await;
    fixture.server.start(fixture.config.clone()).await.unwrap();
    let response = fixture
        .request("POST", "/test", &vec![b'x'; MAX_LOG_BODY + 1], "")
        .await;
    assert!(response.starts_with("HTTP/1.1 201"));
    {
        let logs = fixture.server.logs.lock().unwrap();
        assert!(logs.entries[0].body_truncated);
        assert_eq!(logs.entries[0].body.len(), MAX_LOG_BODY);
    }
    let response = fixture
        .request("POST", "/test", &vec![b'x'; MAX_REQUEST_BODY + 1], "")
        .await;
    assert!(response.starts_with("HTTP/1.1 413"), "{response}");
    assert_eq!(
        fixture.server.logs.lock().unwrap().entries[0]
            .summary
            .status_code,
        413
    );
    for _ in 0..MAX_LOGS {
        fixture.request("GET", "/missing", b"", "").await;
    }
    {
        let logs = fixture.server.logs.lock().unwrap();
        assert_eq!(logs.entries.len(), MAX_LOGS);
        assert_eq!(
            logs.entries.front().unwrap().summary.id,
            (MAX_LOGS + 2) as u64
        );
        assert_eq!(logs.entries.back().unwrap().summary.id, 3);
    }
    fixture.server.stop().await.unwrap();
}

#[tokio::test]
async fn lifecycle_rejects_occupied_port_and_releases_listener_and_loads_config() {
    let fixture = Fixture::new().await;
    let occupied = TcpListener::bind(("127.0.0.1", fixture.config.port))
        .await
        .unwrap();
    assert!(fixture.server.start(fixture.config.clone()).await.is_err());
    assert!(!fixture.server.status().await.running);
    assert!(!fixture.directory.join("config.json").exists());
    drop(occupied);
    fixture.server.start(fixture.config.clone()).await.unwrap();
    assert!(fixture.server.start(fixture.config.clone()).await.is_err());
    fixture.request("POST", "/test", b"", "").await;
    fixture.server.stop().await.unwrap();
    assert!(!fixture.server.status().await.running);
    let released = TcpListener::bind(("127.0.0.1", fixture.config.port))
        .await
        .unwrap();
    drop(released);
    assert_eq!(fixture.server.logs.lock().unwrap().entries.len(), 1);
    let reloaded = TestServer::new(fixture.directory.join("config.json"));
    assert_eq!(reloaded.status().await.config.port, fixture.config.port);
    assert_eq!(reloaded.status().await.config.endpoints[0].status_code, 201);
    assert!(!reloaded.status().await.running);
    fixture.server.start(fixture.config.clone()).await.unwrap();
    fixture.server.stop().await.unwrap();
    fixture.server.stop().await.unwrap();
}

#[test]
fn rejects_invalid_or_conflicting_configuration() {
    let baseline = ServerConfig::default();
    for port in [0, 80, 8765] {
        let mut config = baseline.clone();
        config.port = port;
        assert!(config.validate().is_err());
    }
    for path in [
        "test",
        "//host/test",
        "/test?name=x",
        "/test#fragment",
        "/test path",
        "/中文",
    ] {
        let mut config = baseline.clone();
        config.endpoints[0].path = path.into();
        assert!(config.validate().is_err(), "accepted {path}");
    }
    let mut config = baseline.clone();
    config.endpoints.push(config.endpoints[0].clone());
    assert!(config.validate().is_err());
    config.endpoints[1].method = "POST".into();
    assert!(config.validate().is_err());
    config.endpoints[0].method = "GET".into();
    assert!(config.validate().is_ok());
    config.endpoints[0].response_body = "invalid".into();
    assert!(config.validate().is_err());
    config.endpoints[0].response_body = "{}".into();
    config.endpoints[0].status_code = 100;
    assert!(config.validate().is_err());
}
