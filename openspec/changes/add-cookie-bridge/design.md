## Context

dev-tools 当前是一个 Tauri 2 桌面应用，已有托盘、剪贴板、JSON/SQL/HTTP 解析器三个 Vue 页面，使用 `tauri-plugin-store` 写 JSON 文件做小型持久化。本次新增能力来自一个外部需求：Chrome 扩展抓取目标站点登录态（cookie + localStorage）→ dev-tools 接收并落盘 → Go 业务程序只读消费同一份数据文件。

整体外部架构（dev-tools 仅承担中间一段）：

```
                         HTTP POST /push
┌─────────────┐ ──────────────────────────────▶ ┌─────────────────┐
│ Chrome 扩展 │                                 │ dev-tools (Tauri)│
│  (上游写入) │                                 │  axum + rusqlite │
└─────────────┘                                 └────────┬────────┘
                                                         │ WAL 写
                                                         ▼
                                                  ┌─────────────┐
                                                  │   data.db   │
                                                  │  (WAL 模式) │
                                                  └──────┬──────┘
                                                         │
              ┌──────────────────────────────────────────┤
              │                                          │
              │  HTTP GET /domains                       │ 只读
              │  HTTP GET /domains/:domain               │ (可选)
              ▼                                          ▼
       ┌─────────────┐                           ┌─────────────┐
       │ Go 业务程序 │                           │ Go 业务程序 │
       │ (HTTP 查询) │                           │ (SQLite 直连)│
       └─────────────┘                           └─────────────┘
```

约束：
- SQLite 文件需被独立 Go 进程并发读取，因此必须真 SQLite + WAL，不能用 JSON / tauri-plugin-store
- HTTP 服务必须随托盘生命周期常驻，不能跟随主窗口关闭
- 全本机通信，无鉴权要求

## Goals / Non-Goals

**Goals:**
- 在 `127.0.0.1:8765` 提供 `POST /push`、`GET /health`、`GET /domains`、`GET /domains/:domain` 端点
- SQLite schema 简洁、仅保留每个 (domain, name, path) 的最新状态；写入用事务做"按域名镜像"
- 数据库路径在 Tauri `app_data_dir()` 下，与现有 `sql_history.json` 同目录，便于统一备份和迁移
- 前端 `/cookie-bridge` 页面：左侧域名列表 + 右侧当前 cookie 表，写入后实时刷新
- HTTP 服务启动失败不阻断主程序

**Non-Goals:**
- 不做鉴权 / token 校验（信任本机 127.0.0.1）
- 不做历史版本（只保留每个 key 的最新值）
- 不在 dev-tools 内提供任何修改 / 删除 cookie 的功能（只接收和展示）
- 不向 Chrome 扩展回推任何数据（单向数据流）
- 不替换或迁移现有 JSON / SQL / HTTP Parser 的存储

## Decisions

### D1. 持久化层：rusqlite (bundled) + WAL

- **选择**：`rusqlite` with `bundled` feature
- **替代方案**：
  - `sqlx`：原生 async，但拖入更大的依赖树，且与 Tauri 现有同步代码风格不一致
  - 继续用 JSON：无法被独立 Go 进程并发读取
- **理由**：跨进程读取硬约束 → 必须真 SQLite。`rusqlite` 同步 API 简单，配合 `tokio::task::spawn_blocking` 可融入 axum 的 async 上下文；`bundled` 特性让二进制自带 SQLite，免去 Windows 下的链接问题
- **WAL**：连接初始化时执行 `PRAGMA journal_mode=WAL;`，让多进程并发读写更安全

### D2. HTTP 框架：axum + tower-http(cors)

- **选择**：axum
- **替代方案**：
  - `tiny_http`：依赖更小，但同步阻塞、无 CORS 中间件、未来若加 SSE 不顺手
  - `actix-web`：自带独立 runtime，与 Tauri 的 tokio 不易共存
- **理由**：Tauri 2 内部已用 tokio runtime，axum 共享同一 runtime 边际成本低；`tower-http::cors` 直接套上即可解决扩展端 CORS 问题；社区文档完善

### D3. 写入语义：按域名镜像（事务内 DELETE + INSERT）

- **选择**：每次 `POST /push` 收到 `domain=foo.com` 的载荷时，用事务先 `DELETE FROM cookies WHERE domain='foo.com'`，再 `INSERT` 推送中的所有 cookie；localStorage 同理
- **替代方案**：
  - 纯 UPSERT：浏览器中删除的 cookie 不会同步消失，UI 与浏览器实际状态会偏离
  - 历史版本：表结构复杂，超出本次需求
- **理由**：UI 是"状态视角"，必须等价于浏览器当前状态。镜像写入让"扩展推什么 = 桥就有什么"，无歧义。事务保证不会出现中间态被读取

### D4. 查询通道分层

**前端 → Rust：Tauri command（不走 HTTP）**
- **选择**：前端 `invoke('cookie_bridge_list_domains')` / `invoke('cookie_bridge_get_domain', { domain })`
- **替代方案**：前端直接 `fetch('http://127.0.0.1:8765/domains')`
- **理由**：前端是 dev-tools 内部组件，走 Tauri IPC 零网络开销；HTTP 端点面向外部进程（Go 应用），职责分离

**Go 应用 → HTTP 查询端点（不走 SQLite）**
- **选择**：Go 应用通过 `GET /domains` 和 `GET /domains/:domain` 查询数据
- **替代方案**：Go 应用直接只读打开 SQLite 文件
- **理由**：HTTP 查询让 Go 侧零依赖（无需 SQLite 驱动、无需关心 WAL 兼容性、无需知道 `data.db` 的绝对路径），降低集成成本；SQLite 直连作为备选方案保留

### D5. 实时刷新：Tauri Event

- **选择**：`POST /push` 处理函数写完 DB 后调用 `app_handle.emit("cookie-bridge:updated", domain)`，前端 `listen` 后重新 invoke 拉取
- **替代方案**：
  - 前端轮询：实现简单，但有 ~3s 延迟
  - SSE：需要前端走 HTTP 协议，与 D4 决策矛盾
- **理由**：Tauri Event 是为这个场景设计的，零额外依赖、毫秒级延迟

### D6. 端口冲突处理：固定 8765 + 优雅降级

- **选择**：固定监听 8765；`TcpListener::bind` 失败时记录错误日志、发托盘通知（"Cookie 桥不可用：端口被占"），但 `setup()` 仍返回 `Ok(())`
- **替代方案**：
  - 自动换端口：扩展端不知道新端口，需要再加发现机制，复杂度爆炸
  - 启动失败直接退出：cookie-bridge 是边缘功能，不应拖死整个 dev-tools
- **理由**：扩展端硬编码 8765 → 端口必须固定。优雅降级让其他功能（解析器、剪贴板）继续可用

### D7. 数据库路径

- **选择**：`app.path().app_data_dir()?.join("data.db")`
- **替代方案**：硬编码 `%LOCALAPPDATA%\CookieBridge\data.db`
- **理由**：复用 dev-tools 的 identifier（避免命名分裂），与现有 `sql_history.json` 同目录便于运维。Go 程序通过环境变量或配置读取此绝对路径

### D8. 模块组织

```
src-tauri/src/cookie_bridge/
├── mod.rs        ── 对外暴露 init(app_handle) -> Result<()>
├── db.rs         ── Connection 持有（Mutex<Connection>）+ schema 迁移 + 镜像写入
├── http.rs       ── axum router + handlers，handler 中调用 db.rs
└── commands.rs   ── #[tauri::command] list_domains / get_domain_cookies
```

DB 连接放进 Tauri 的 `State<Mutex<Connection>>`，HTTP handler 通过 axum 的 `State` extension 拿到同一个 `Arc<Mutex<Connection>>`。

## Risks / Trade-offs

- **[Mutex<Connection> 串行化所有写入]** → 在本场景下 push 频次低（用户登录态变化才推），写吞吐不是瓶颈；如未来需要并发，可换 `r2d2` 连接池
- **[rusqlite 同步 API 在 axum handler 中阻塞 runtime]** → 所有 DB 调用必须用 `tokio::task::spawn_blocking` 包装；约定写在 `db.rs` 内部
- **[Go 端读取 WAL 的兼容性]** → modernc.org/sqlite 支持 WAL 读取，但需要 `-shm` 文件可访问；Windows 下若 dev-tools 持有独占锁可能导致 Go 端报 `database is locked`。**Mitigation**：dev-tools 使用 `OpenFlags::SQLITE_OPEN_READ_WRITE | SQLITE_OPEN_CREATE`，不加 EXCLUSIVE；保持 WAL 模式
- **[端口 8765 与其他服务冲突]** → D6 已优雅降级；用户可在文档中看到日志和托盘通知
- **[Chrome 扩展端 CORS 限制]** → tower-http::cors 设为 `Any` origin（仅监听 127.0.0.1，无外部攻击面），扩展 manifest 仍需声明 `host_permissions: ["http://127.0.0.1:8765/*"]`
- **[/push 体积过大被滥用]** → axum 默认 body limit 2MB，对 cookie 推送绰绰有余；可保留默认值

## Migration Plan

不涉及现有数据迁移：
- 新模块、新表、新页面、新依赖，均为纯新增
- 现有 `parse_history.json` / `sql_history.json` / `json_parser_history.json` 不变
- 回滚策略：移除 `cookie_bridge` 模块、Cargo 依赖、前端路由项即可；`data.db` 文件留在数据目录不影响其他功能

## Open Questions

- 是否需要在前端页面提供"导出当前所有数据为 JSON"的辅助功能？—— 暂不做，等真实使用反馈
