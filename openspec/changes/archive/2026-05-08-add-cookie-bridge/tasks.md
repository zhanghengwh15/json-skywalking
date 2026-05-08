## 1. 依赖与工程准备

- [x] 1.1 在 `src-tauri/Cargo.toml` 中加入 `rusqlite = { version = "0.31", features = ["bundled"] }`
- [x] 1.2 在 `src-tauri/Cargo.toml` 中加入 `axum = "0.7"`、`tower-http = { version = "0.5", features = ["cors"] }`
- [x] 1.3 在 `src-tauri/Cargo.toml` 中加入 `tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync"] }`
- [x] 1.4 运行一次 `cargo check`，确认依赖能解析、无版本冲突

## 2. 数据库层 (`cookie_bridge/db.rs`)

- [x] 2.1 新建文件 `src-tauri/src/cookie_bridge/db.rs`
- [x] 2.2 定义 `pub struct Db { conn: Arc<Mutex<rusqlite::Connection>> }`，提供 `Db::open(path: &Path) -> rusqlite::Result<Self>`
- [x] 2.3 在 `open` 中执行 `PRAGMA journal_mode=WAL;` 与 `PRAGMA foreign_keys=ON;`
- [x] 2.4 定义并执行 schema 迁移 SQL：建 `cookies` 与 `local_storage` 两张表，主键和字段按 spec 中定义
- [x] 2.5 实现 `Db::push(&self, payload: &PushPayload) -> rusqlite::Result<(usize, usize)>`：在事务内先按 domain 删除旧记录，再批量 insert，返回写入的 cookie 数与 localStorage 条目数
- [x] 2.6 实现 `Db::list_domains(&self) -> rusqlite::Result<Vec<String>>`：联合 cookies 与 local_storage，按 MAX(updated_at) 倒序去重
- [x] 2.7 实现 `Db::get_domain(&self, domain: &str) -> rusqlite::Result<DomainSnapshot>`：返回该域所有 cookie 与 localStorage 条目
- [x] 2.8 所有公开方法均使用同步 API，调用方负责用 `spawn_blocking` 包装

## 3. HTTP 服务 (`cookie_bridge/http.rs`)

- [x] 3.1 新建文件 `src-tauri/src/cookie_bridge/http.rs`
- [x] 3.2 定义请求/响应结构体：`PushPayload`、`PushCookie`、`PushLocalStorage`、`PushResponse`，均派生 `Serialize` / `Deserialize`
- [x] 3.3 构建 axum `Router`：`POST /push`、`GET /health`，挂载 `tower_http::cors::CorsLayer::permissive()`
- [x] 3.4 实现 `health_handler` 直接返回 `"ok"`
- [x] 3.5 实现 `push_handler`：从 `State` 取 `Db` 与 `AppHandle`；用 `spawn_blocking` 调 `db.push(...)`；成功后 `app_handle.emit("cookie-bridge:updated", &payload.domain)`；返回 `PushResponse { ok, cookies, local_storage }`
- [x] 3.6 新增 `GET /domains` 端点：实现 `list_domains_handler`，`spawn_blocking` 调 `db.list_domains()`，返回 JSON 数组
- [x] 3.7 新增 `GET /domains/:domain` 端点：实现 `get_domain_handler`，`spawn_blocking` 调 `db.get_domain()`，返回 `DomainSnapshot` JSON
- [x] 3.8 实现 `pub async fn serve(addr: SocketAddr, state: AppState) -> Result<(), io::Error>`：`TcpListener::bind` + `axum::serve(...)`；调用方判断 bind 错误

## 4. Tauri commands (`cookie_bridge/commands.rs`)

- [x] 4.1 新建文件 `src-tauri/src/cookie_bridge/commands.rs`
- [x] 4.2 实现 `#[tauri::command] cookie_bridge_list_domains(state: State<'_, Db>) -> Result<Vec<String>, String>`，内部 `spawn_blocking` 包 `db.list_domains()`
- [x] 4.3 实现 `#[tauri::command] cookie_bridge_get_domain(domain: String, state: State<'_, Db>) -> Result<DomainSnapshot, String>`，同样 `spawn_blocking` 包装
- [x] 4.4 让返回结构体派生 `Serialize`，字段使用 camelCase（`#[serde(rename_all = "camelCase")]`）以贴合前端

## 5. 模块入口与启动 (`cookie_bridge/mod.rs`)

- [x] 5.1 新建文件 `src-tauri/src/cookie_bridge/mod.rs`，声明 `pub mod db; pub mod http; pub mod commands;`
- [x] 5.2 实现 `pub fn init(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>>`：解析 `app.path().app_data_dir()`；确保目录存在；`Db::open(...)`；将 `Db` 注册为 `app.manage(...)`；spawn 一个 tokio 任务调用 `http::serve(...)`
- [x] 5.3 在 `init` 中：若 HTTP `serve` 在 `bind` 阶段失败，记录 `eprintln!`、调用 tray 通知 API 弹"Cookie 桥不可用：端口 8765 被占"，返回 `Ok(())`（不阻断主程序）
- [x] 5.4 HTTP 服务的 `AppState` 同时持有 `Db` 与 `AppHandle` 的克隆，用于 emit 事件

## 6. 接入主程序 (`lib.rs`)

- [x] 6.1 在 `src-tauri/src/lib.rs` 顶部加 `mod cookie_bridge;`
- [x] 6.2 在 `invoke_handler!` 宏中追加 `cookie_bridge::commands::cookie_bridge_list_domains` 与 `cookie_bridge::commands::cookie_bridge_get_domain`
- [x] 6.3 在 `setup` 闭包中调用 `cookie_bridge::init(app)`，错误仅打印日志、不返回 Err

## 7. 前端页面与路由

- [x] 7.1 新建 `src/views/CookieBridge.vue`：左右两栏布局；左侧列表绑定 `domains`；右侧两个表格分别渲染 `currentDomain.cookies` 与 `currentDomain.localStorage`
- [x] 7.2 在 `onMounted` 调用 `invoke('cookie_bridge_list_domains')` 初始化列表；默认选中第一项并加载详情
- [x] 7.3 用 `listen('cookie-bridge:updated', ...)` 订阅事件：收到事件后重新调 `list_domains`；若事件 payload 等于当前选中域则同时刷新详情
- [x] 7.4 处理空状态：`domains` 为空时显示"等待 Chrome 扩展推送数据"提示
- [x] 7.5 在 `src/router/index.ts` 加入 `{ path: '/cookie-bridge', name: 'CookieBridge', component: () => import('../views/CookieBridge.vue') }`

## 8. 验证

- [ ] 8.1 `npm run tauri dev` 启动 dev-tools，检查日志看到 HTTP 服务在 8765 启动成功
- [ ] 8.2 `curl http://127.0.0.1:8765/health` 返回 `ok`
- [ ] 8.3 `curl -X POST http://127.0.0.1:8765/push -H "Content-Type: application/json" -d '{"domain":"test.com","cookies":[{"domain":"test.com","name":"sid","value":"abc","path":"/","expires":0,"secure":0,"http_only":0}],"local_storage":null,"ts":1700000000000}'` 返回 `{"ok":true,"cookies":1,"local_storage":0}`
- [ ] 8.4 在浏览器访问 `/cookie-bridge`，验证 `test.com` 出现在左侧列表，右侧显示 `sid=abc`
- [ ] 8.5 再次推送同域但只携带 `uid`，验证 UI 自动刷新且 `sid` 已不存在（镜像语义生效）
- [ ] 8.6 `curl http://127.0.0.1:8765/domains` 返回 `["test.com"]`
- [ ] 8.7 `curl http://127.0.0.1:8765/domains/test.com` 返回包含 `cookies` 和 `localStorage` 的 JSON
- [ ] 8.8 用 SQLite 客户端打开 `app_data_dir/data.db`，确认 `cookies` 与 `local_storage` 表存在、WAL 模式生效（同目录有 `data.db-wal` 和 `data.db-shm`）
- [ ] 8.9 在 dev-tools 运行时启动一个 Go 小程序（或用 `sqlite3` CLI）只读打开同一文件 `SELECT * FROM cookies`，确认能读出数据且不报锁错
- [ ] 8.10 故意先用 `python -m http.server 8765` 占住端口，再启动 dev-tools，确认主程序仍能启动、托盘正常、出现"端口被占"通知或日志
