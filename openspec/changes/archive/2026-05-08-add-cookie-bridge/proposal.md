## Why

外部 Go 业务程序需要访问目标站点登录后的 cookie 与 localStorage，但浏览器中的会话信息无法被外部进程直接读取。需要一座桥：浏览器（Chrome 扩展）抓取会话数据 → 桥接服务接收并落盘到一个独立进程也能读的存储 → Go 程序只读消费。dev-tools 作为常驻托盘应用，是承载这个桥的合适位置。

## What Changes

- 在 dev-tools (Tauri) 中新增 `cookie-bridge` 模块，启动本地 HTTP 服务（`127.0.0.1:8765`），接收 Chrome 扩展推送的 cookie / localStorage 数据
- 引入 SQLite（rusqlite + WAL 模式）作为持久化存储，路径位于 Tauri 应用数据目录下的 `data.db`，供 Go 进程以只读方式读取同一文件
- 写入语义为"按域名镜像"：每次推送以事务方式整体替换该域的 cookies / localStorage 集合
- 新增 Vue 页面 `/cookie-bridge`，按域名分组展示当前 cookie 状态（左侧域名列表 + 右侧 cookie 表）
- 写入完成后通过 Tauri Event 通知前端实时刷新
- HTTP 服务启动失败（端口被占）时，发送托盘通知并记录日志，但 Tauri 主进程仍然正常启动，cookie-bridge 能力不可用
- **新增 HTTP 查询端点**：`GET /domains` 返回所有域名列表，`GET /domains/:domain` 返回指定域名的 cookie 与 localStorage 详情，供内部 Go 应用直接通过 HTTP 消费数据，无需接触 SQLite 文件

## Capabilities

### New Capabilities
- `cookie-bridge`: 通过本地 HTTP 接收浏览器扩展推送的 cookie / localStorage，按域名镜像式持久化到共享 SQLite，并提供 UI 状态视图与跨进程只读访问

### Modified Capabilities
<!-- 无现有 spec 需要修改 -->

## Impact

- **Rust 依赖新增**：`rusqlite`（bundled 特性）、`axum`、`tower-http`（cors）、`tokio`（rt-multi-thread + macros 特性，用于 spawn server）
- **Rust 模块新增**：`src-tauri/src/cookie_bridge/{mod.rs, db.rs, http.rs, commands.rs}`
- **Rust 启动流程变更**：`lib.rs` 的 `setup()` 中新增 cookie_bridge 初始化（开 DB、起 HTTP server）
- **前端新增**：`src/views/CookieBridge.vue`，路由表加入 `/cookie-bridge`
- **数据文件新增**：`%APPDATA%\com.dev-tools.app\data.db` 及其 `-wal` / `-shm` 伴生文件（路径以 Tauri identifier 为准）
- **网络监听**：新增对 `127.0.0.1:8765` 的本地端口占用
- **跨进程契约**：Go 程序需以 SQLite WAL 兼容方式（如 `modernc.org/sqlite`）只读打开同一 `data.db`，schema 由本变更定义
- **不影响**：现有 JSON / SQL / HTTP Parser 模块、tauri-plugin-store 中的解析历史
