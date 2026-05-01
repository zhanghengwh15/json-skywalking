# Cookie Bridge 模块文档

## 概述

Cookie Bridge 是 dev-tools 的一个子模块，负责接收来自上游（Chrome 扩展）的 cookie / localStorage 数据，持久化到 SQLite，并为下游（Go 业务程序、前端 UI）提供查询能力。

```
┌─────────────┐  HTTP POST /push  ┌─────────────────┐  WAL 写  ┌─────────────┐
│ Chrome 扩展 │ ────────────────▶ │  dev-tools      │ ──────▶ │  data.db    │
│  (上游写入) │                   │  cookie_bridge  │         │ (WAL 模式)  │
└─────────────┘                   └─────────────────┘         └──────┬──────┘
                                                                      │
                     ┌──────────────────────────────────────────────┤
                     │                                              │
                     ▼                                              ▼
              ┌─────────────┐                               ┌─────────────┐
              │ 前端 Vue UI │                               │ Go 业务程序 │
              │ (下游查询)  │                               │ (下游查询)  │
              └─────────────┘                               └─────────────┘
```

---

## 上游写入：HTTP 推送接口

### 服务地址

- **Host**: `127.0.0.1:8765`（仅本机，不对外暴露）
- **CORS**: 已配置为全开放（`CorsLayer::permissive()`），允许 Chrome 扩展跨源调用

### 端点

#### `GET /health` — 健康检查

**响应**：
```
HTTP/1.1 200 OK
Body: ok
```

#### `POST /push` — 推送 cookie / localStorage 数据

**请求头**：
```
Content-Type: application/json
```

**请求体**：
```json
{
  "domain": "example.com",
  "cookies": [
    {
      "domain": "example.com",
      "name": "sid",
      "value": "abc123",
      "path": "/",
      "expires": 1700000000,
      "secure": 1,
      "http_only": 1
    }
  ],
  "local_storage": [
    {
      "key": "user_id",
      "value": "42"
    }
  ],
  "ts": 1700000000000
}
```

**字段说明**：

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `domain` | string | 是 | 数据所属的域名 |
| `cookies` | array / null | 否 | cookie 列表，`null` 表示不修改该域的 cookies |
| `local_storage` | array / null | 否 | localStorage 条目列表，`null` 表示不修改 |
| `ts` | number | 是 | 推送时间戳（毫秒），用于排序和去重 |

**Cookie 条目字段**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `domain` | string | cookie 所属的域名 |
| `name` | string | cookie 名称 |
| `value` | string | cookie 值 |
| `path` | string | cookie 路径 |
| `expires` | number | 过期时间戳（秒） |
| `secure` | number | 是否仅 HTTPS 传输（0/1） |
| `http_only` | number | 是否禁止 JS 访问（0/1） |

**LocalStorage 条目字段**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | string | 键名 |
| `value` | string | 值 |

**写入语义 — 按域名镜像**：

每次推送以**事务**方式执行：
1. `DELETE FROM cookies WHERE domain = ?`
2. `INSERT` 推送中携带的所有 cookie
3. `DELETE FROM local_storage WHERE domain = ?`
4. `INSERT` 推送中携带的所有 localStorage

这意味着：推送什么，DB 里就有什么。浏览器中已删除的 cookie 不会残留。

**成功响应**（HTTP 200）：
```json
{
  "ok": true,
  "cookies": 1,
  "local_storage": 1
}
```

**错误响应**（HTTP 400 / 500）：
```json
{
  "error": "domain is required"
}
```

**写入后事件通知**：

推送成功且事务提交后，Rust 端会广播 Tauri 事件：
```
event: "cookie-bridge:updated"
payload: "example.com"   // 被更新的域名
```

前端页面监听此事件实现实时刷新。

---

## 下游查询

### 方式一：前端 UI（Tauri Command）

前端通过 `invoke` 调用 Rust 命令，走 Tauri IPC，不经过 HTTP。

#### `cookie_bridge_list_domains`

返回所有出现过的域名列表，按各域最新更新时间倒序排列。

```typescript
import { invoke } from '@tauri-apps/api/core'

const domains: string[] = await invoke('cookie_bridge_list_domains')
// ["example.com", "test.com", ...]
```

#### `cookie_bridge_get_domain`

返回指定域名的 cookie 和 localStorage 详情。

```typescript
const snapshot = await invoke('cookie_bridge_get_domain', { domain: 'example.com' })
```

**返回结构**：
```json
{
  "cookies": [
    {
      "domain": "example.com",
      "name": "sid",
      "path": "/",
      "value": "abc123",
      "expires": 1700000000,
      "secure": 1,
      "httpOnly": 1,
      "updatedAt": 1700000000000
    }
  ],
  "localStorage": [
    {
      "domain": "example.com",
      "key": "user_id",
      "value": "42",
      "updatedAt": 1700000000000
    }
  ]
}
```

> 注意：Rust 端返回 camelCase 字段名（通过 `#[serde(rename_all = "camelCase")]` 转换）。

---

### 方式二：外部进程（直接读 SQLite）

Go / Python / 其他程序可以直接以**只读**方式打开同一个 SQLite 文件。

#### 数据库文件路径

```
%APPDATA%\com.dev-tools.app\data.db          (Windows)
~/Library/Application Support/com.dev-tools.app/data.db   (macOS)
~/.local/share/com.dev-tools.app/data.db                  (Linux)
```

> 实际路径由 Tauri 的 `app.path().app_data_dir()` 决定，与 `tauri.conf.json` 中的 `identifier` 字段相关。

#### 连接方式

**Go 示例**（使用 `modernc.org/sqlite`）：
```go
import (
    "database/sql"
    _ "modernc.org/sqlite"
)

db, err := sql.Open("sqlite", "file:/path/to/data.db?mode=ro")
if err != nil {
    log.Fatal(err)
}
defer db.Close()

rows, err := db.Query("SELECT domain, name, value FROM cookies WHERE domain = ?", "example.com")
```

**关键参数**：`mode=ro`（只读模式），避免与 dev-tools 的写入连接产生锁冲突。

#### Schema

**cookies 表**：
```sql
CREATE TABLE cookies (
    domain     TEXT NOT NULL,
    name       TEXT NOT NULL,
    path       TEXT NOT NULL,
    value      TEXT NOT NULL,
    expires    INTEGER NOT NULL DEFAULT 0,
    secure     INTEGER NOT NULL DEFAULT 0,
    http_only  INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (domain, name, path)
);
```

**local_storage 表**：
```sql
CREATE TABLE local_storage (
    domain     TEXT NOT NULL,
    key        TEXT NOT NULL,
    value      TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (domain, key)
);
```

#### WAL 模式说明

数据库以 WAL（Write-Ahead Logging）模式运行，同目录下会有两个伴生文件：
- `data.db-wal` — 未提交的写入日志
- `data.db-shm` — 共享内存文件

外部进程读取时：
- 确保这三个文件在同一目录且可读
- 使用支持 WAL 的 SQLite 驱动（如 `modernc.org/sqlite`）
- 以只读模式打开，不会阻塞 dev-tools 的写入

---

## 模块文件结构

```
cookie_bridge/
├── mod.rs        — 模块入口，初始化 DB + 启动 HTTP 服务
├── db.rs         — SQLite 操作（schema、push、list_domains、get_domain）
├── http.rs       — axum HTTP 服务（/health、/push）
├── commands.rs   — Tauri command（供前端调用）
└── README.md     — 本文档
```

---

## 错误处理

| 场景 | 行为 |
|------|------|
| 端口 8765 被占用 | 记录错误日志，发送托盘通知，主程序继续启动 |
| 推送缺少 `domain` | 返回 HTTP 400 |
| DB 写入失败 | 事务回滚，返回 HTTP 500，不发送更新事件 |
| 查询不存在的域名 | 返回 `{ cookies: [], localStorage: [] }`，不抛错 |
