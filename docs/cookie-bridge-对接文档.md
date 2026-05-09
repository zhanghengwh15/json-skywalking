# Cookie Bridge 对接文档

Cookie Bridge 是 dev-tools 的核心子模块，负责接收来自上游（Chrome 扩展等）的 cookie / localStorage 数据，持久化到 SQLite，并通过 HTTP / Tauri Command 为下游（前端 UI、Go 业务程序、CLI 等）提供查询能力。

数据模型已升级为 **Domain 关系模型**：所有 cookies / localStorage 通过 `domain_id` 外键关联到 `domains` 表，删除域时级联清理。

```
┌─────────────┐  HTTP POST /push  ┌─────────────────┐  WAL 写  ┌─────────────┐
│ Chrome 扩展 │ ────────────────▶ │  dev-tools      │ ──────▶ │  data.db    │
│  (上游写入) │                   │  cookie_bridge  │         │ (WAL 模式)  │
└─────────────┘                   └─────────────────┘         └──────┬──────┘
                                                                      │
                     ┌────────────────────────────────────────────────┤
                     │                                                │
                     ▼                                                ▼
              ┌─────────────┐                               ┌─────────────────┐
              │ 前端 Vue UI │                               │ Go / CLI / 其他 │
              │ (Tauri IPC) │                               │ (HTTP / 只读 DB)│
              └─────────────┘                               └─────────────────┘
```

---

## 一、上游写入：HTTP 推送接口

### 服务地址

- **Host**：`127.0.0.1:8765`（仅本机绑定，不对外暴露）
- **CORS**：`CorsLayer::permissive()`，允许 Chrome 扩展跨源调用
- **请求体上限**：10 MB（`RequestBodyLimitLayer`）

### `GET /health` — 健康检查

```
HTTP/1.1 200 OK
Body: ok
```

### `POST /push` — 推送 cookie / localStorage 数据

**请求头**：`Content-Type: application/json`

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
      "secure": true,
      "httpOnly": true
    }
  ],
  "local_storage": [
    { "key": "user_id", "value": "42" }
  ],
  "ts": 1700000000000
}
```

**顶层字段**：

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `domain` | string | 是 | 数据所属域名；为空返回 HTTP 400 |
| `cookies` | array / null | 否 | cookie 列表，`null` 或省略 = 不修改该域 cookies |
| `local_storage` | array / object / null | 否 | 既支持数组形式，也支持 `{ key: value }` 对象形式（兼容上游历史格式）；`null` 或省略 = 不修改 |
| `ts` | number | 否 | 推送时间戳（毫秒）；缺省时由服务端用当前时间填充 |

**Cookie 条目字段**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `domain` | string | cookie 所属域名 |
| `name` | string | cookie 名称 |
| `value` | string | cookie 值 |
| `path` | string | cookie 路径，缺省为 `""` |
| `expires` / `expirationDate` | number | 过期时间戳（秒），可选；二者别名等价 |
| `secure` | bool / 0 \| 1 | 是否仅 HTTPS 传输；同时接受布尔与整数 |
| `httpOnly` / `http_only` | bool / 0 \| 1 | 是否禁止 JS 访问；同时接受布尔与整数 |

> 服务端通过自定义反序列化器同时接受 `true/false` 与 `0/1`，以适配不同上游扩展的写法。

**LocalStorage 条目字段**：

| 字段 | 类型 | 说明 |
|------|------|------|
| `key` | string | 键名 |
| `value` | string | 值 |

或者直接传对象 `{ "user_id": "42", "token": "xxx" }`，服务端会自动按 key/value 拆开。

**写入语义 — 按域名镜像**：

每次推送以**事务**方式执行（基于 `domain_id`）：

1. 调用 `INSERT OR IGNORE` 确保 `domains` 表中存在该 domain 行，取得 `domain_id`
2. 若提供 `cookies`：`DELETE FROM cookies WHERE domain_id = ?`，再批量 `INSERT`
3. 若提供 `local_storage`：`DELETE FROM local_storage WHERE domain_id = ?`，再批量 `INSERT`

**未提供的字段不会被清空**（例如只推 cookies 不会清掉 localStorage）。提供了的字段则完全镜像浏览器现状。

**成功响应**（HTTP 200）：

```json
{ "ok": true, "cookies": 1, "local_storage": 1 }
```

**错误响应**：

```json
{ "error": "domain is required" }       // 400
{ "error": "db error: ..." }            // 500
```

**写入后事件通知**：

事务提交后，Rust 端会通过 Tauri 广播事件：

```
event:   "cookie-bridge:updated"
payload: "example.com"   // 被更新的域名字符串
```

前端通过 `listen('cookie-bridge:updated', ...)` 实现实时刷新。

---

## 二、HTTP 查询接口

> 全部位于同一个 `127.0.0.1:8765` 服务上。返回体存在两种风格：
> - 早期接口（`/domains`、`/domains/:domain`）：直接返回数据 / `{ error }`
> - 新接口（`/api/domains/...`）：统一封装为 `{ success, data, message }`

### 2.1 旧接口（保留兼容）

#### `GET /domains`

返回所有**已写入过 cookies 或 localStorage** 的域名记录列表，按各域最新 `updated_at` 倒序。

返回：`Domain[]`，每项形如：

```json
{
  "id": 12,
  "domainName": "example.com",
  "urls": "[\"https://example.com\"]",
  "description": null,
  "createdAt": "2026-05-01 12:00:00",
  "updatedAt": "2026-05-09 18:30:00"
}
```

#### `GET /domains/:domain`

按 `domain_name` 查询单域快照。

返回 `DomainSnapshot`：

```json
{
  "domain": { "id": 12, "domainName": "example.com", "...": "..." },
  "cookies": [
    {
      "domainName": "example.com",
      "name": "sid",
      "path": "/",
      "value": "abc123",
      "expires": 1700000000,
      "secure": true,
      "httpOnly": true,
      "updatedAt": 1700000000000
    }
  ],
  "localStorage": [
    {
      "domainName": "example.com",
      "key": "user_id",
      "value": "42",
      "updatedAt": 1700000000000
    }
  ]
}
```

> 字段统一为 **camelCase**（`#[serde(rename_all = "camelCase")]`）。`domain` 不存在时为 `null`，`cookies` / `localStorage` 永远是数组。

### 2.2 Domain CRUD（`/api/domains`）

详细字段说明见 [`docs/domain-management-对接文档.md`](domain-management-对接文档.md)。简表：

| 方法 | 路径 | 作用 |
|------|------|------|
| `GET` | `/api/domains` | 列表 |
| `POST` | `/api/domains` | 新建（必填 `domainName`；`urls` 为 JSON 数组字符串） |
| `GET` | `/api/domains/:id` | 单条 |
| `PUT` | `/api/domains/:id` | 更新（部分字段） |
| `DELETE` | `/api/domains/:id` | 删除（级联清掉关联 cookies / localStorage） |
| `GET` | `/api/domains/match?url=...` | URL 匹配 |

`/api/domains/match` 的命中规则：

1. 先解析 URL 的 host
2. 精确匹配 `domain_name`
3. 回退：遍历每条 domain 的 `urls` JSON 数组，提取每个 pattern 的 host，去掉 `*.` 前缀后做后缀匹配（`host.ends_with(pattern_host_clean)`）

返回：

```json
{
  "success": true,
  "data": {
    "domain": { "id": 12, "domainName": "example.com", "...": "..." },
    "cookies": [ ... ],
    "local_storage": [ ... ]
  }
}
```

未命中时 `domain` 为 `null`，`cookies` / `local_storage` 为空数组（仍是 `success: true`）。

### 2.3 Task Branch Group（`/api/task-branch-groups`）

由 `task_branch_group` 模块挂载，详见 [`docs/task-branch-group-对接文档.md`](task-branch-group-对接文档.md)。

---

## 三、前端 Tauri Commands

前端通过 `invoke` 调用以下命令，走 IPC，不经过 HTTP。

### Cookie Bridge 查询

| Command | 参数 | 返回 | 说明 |
|---------|------|------|------|
| `cookie_bridge_list_domains` | — | `Domain[]` | 列出所有有数据的域，按最近写入时间倒序 |
| `cookie_bridge_get_domain` | `{ domain: string }` | `DomainSnapshot` | 按域名取快照 |
| `cookie_bridge_delete_domain` | `{ domain: string }` | `[number, number]` | 按域名删除，返回 `[cookies, localStorage]` 删除条数 |
| `cookie_bridge_set_debug_mode` | `{ enabled: boolean }` | `void` | 切换详细日志 |
| `cookie_bridge_get_debug_mode` | — | `boolean` | 读取 debug 状态 |

### Domain CRUD

| Command | 参数 | 返回 |
|---------|------|------|
| `domain_list` | — | `Domain[]` |
| `domain_create` | `{ payload: CreateDomain }` | `Domain` |
| `domain_get` | `{ id: number }` | `Domain` |
| `domain_update` | `{ id: number, payload: UpdateDomain }` | `Domain` |
| `domain_delete` | `{ id: number }` | `boolean` |
| `domain_match_url` | `{ url: string }` | `MatchResult` |

```typescript
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const domains = await invoke<Domain[]>('cookie_bridge_list_domains')
const snapshot = await invoke<DomainSnapshot>('cookie_bridge_get_domain', { domain: 'example.com' })

await listen<string>('cookie-bridge:updated', (e) => {
  console.log('域名已更新:', e.payload)
})
```

---

## 四、外部进程：直接读 SQLite

Go / Python / 其他程序可以直接以**只读**方式打开同一个 SQLite 文件。

### 数据库路径

由 Tauri 的 `app.path().app_data_dir()` 决定，与 `tauri.conf.json` 的 `identifier`（`com.tauri.devtools`）相关：

| OS | 路径 |
|----|------|
| Windows | `%APPDATA%\com.tauri.devtools\data.db` |
| macOS | `~/Library/Application Support/com.tauri.devtools/data.db` |
| Linux | `~/.local/share/com.tauri.devtools/data.db` |

同目录下还会有日志文件 `cookie-bridge.log`。

### Go 示例（`modernc.org/sqlite`）

```go
import (
    "database/sql"
    _ "modernc.org/sqlite"
)

db, _ := sql.Open("sqlite", "file:/path/to/data.db?mode=ro")
defer db.Close()

rows, _ := db.Query(`
    SELECT c.name, c.value, c.path
    FROM cookies c
    JOIN domains d ON c.domain_id = d.id
    WHERE d.domain_name = ?`, "example.com")
```

**关键参数**：`mode=ro`，避免与 dev-tools 写入连接产生锁冲突。

### Schema（当前版本）

```sql
CREATE TABLE domains (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    domain_name TEXT NOT NULL UNIQUE,
    urls        TEXT,                              -- JSON 数组字符串
    description TEXT,
    created_at  DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at  DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE cookies (
    domain_id  INTEGER NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    path       TEXT NOT NULL,
    value      TEXT NOT NULL,
    expires    INTEGER NOT NULL DEFAULT 0,
    secure     INTEGER NOT NULL DEFAULT 0,
    http_only  INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (domain_id, name, path)
);

CREATE TABLE local_storage (
    domain_id  INTEGER NOT NULL REFERENCES domains(id) ON DELETE CASCADE,
    key        TEXT NOT NULL,
    value      TEXT NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (domain_id, key)
);

CREATE TABLE task_branch_group ( ... );  -- 见 task-branch-group 文档
```

> **Schema 迁移**：旧版本 `cookies` 表直接含有 `domain TEXT` 列。启动时若检测到旧 schema，会 **`DROP TABLE cookies` / `local_storage`**（**会丢弃历史数据**）后重建。生产环境从旧版本升级前请提前导出。

### WAL 模式

数据库以 WAL 运行，同目录会出现：
- `data.db-wal` — 未提交的写入日志
- `data.db-shm` — 共享内存文件

外部进程读取注意：
- 三个文件需在同一目录且可读
- 使用支持 WAL 的 SQLite 驱动
- `mode=ro` 不会阻塞 dev-tools 的写入

---

## 五、模块文件结构

```
src-tauri/src/cookie_bridge/
├── mod.rs        — 模块入口；初始化日志、打开 DB、启动 HTTP 服务
├── db.rs         — SQLite schema、迁移、push、域 CRUD、URL 匹配
├── http.rs       — axum HTTP 服务（/health、/push、/domains、/api/domains/*）
└── commands.rs   — Tauri commands（供前端调用）
```

---

## 六、错误处理与运维

| 场景 | 行为 |
|------|------|
| 端口 8765 被占用 | 记录错误日志 + 通过 `tauri://notification` 事件提示，主程序继续启动 |
| 推送缺少 `domain` | 返回 HTTP 400 `{ "error": "domain is required" }` |
| DB 写入失败 | 事务回滚，返回 HTTP 500，不发送 `cookie-bridge:updated` 事件 |
| 查询不存在的域名 | 返回 `{ domain: null, cookies: [], localStorage: [] }`，不抛错 |
| `POST /api/domains` 触发 UNIQUE 冲突 | 返回 HTTP 409 `{ "message": "domain already exists" }` |
| 升级时检测到旧 schema | 自动重建 `cookies` / `local_storage`，**丢弃历史数据** |

**调试日志**：`cookie_bridge_set_debug_mode(true)` 或对应 HTTP 接口可开启详细日志，输出每条 cookie / localStorage 的写入细节到 `cookie-bridge.log`。
