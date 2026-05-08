## MODIFIED Requirements

### Requirement: cookie / localStorage 推送接收与镜像写入
系统 SHALL 接受形如 `{"domain": <string>, "cookies": [<cookie>...] | null, "local_storage": [<entry>...] | null, "ts": <number>}` 的 JSON POST 请求，并以"按域名镜像"的事务方式写入 SQLite。一次推送 MUST 在单个事务内：先根据 `payload.domain` upsert `domains` 表获取 `domain_id`，再 `DELETE FROM cookies WHERE domain_id=?`、`DELETE FROM local_storage WHERE domain_id=?`，最后 `INSERT` 推送中携带的所有条目。

#### Scenario: 首次推送某域的 cookie
- **WHEN** 客户端 POST `/push` 携带 `{"domain":"example.com", "cookies":[{"domain":"example.com","name":"sid","value":"abc","path":"/","expires":0,"secure":0,"http_only":0}], "local_storage":null, "ts":1700000000000}`
- **THEN** 服务返回 HTTP 200，响应体 `{"ok":true,"cookies":1,"local_storage":0}`；`domains` 表中存在 `domain_name='example.com'` 的行；`cookies` 表中存在 `(domain_id=<该域id>, name='sid', path='/', value='abc')`

#### Scenario: 同域再次推送时整体替换
- **WHEN** 上一个场景已写入 `sid=abc`，紧接着客户端再次 POST 携带同域但仅包含 `{"name":"uid","value":"u1",...}` 的 cookies 列表
- **THEN** 写入完成后，`example.com` 域下只剩 `uid=u1`，原有的 `sid` 行被事务一并删除

#### Scenario: 推送中 cookies 为 null
- **WHEN** 客户端 POST 携带 `"cookies": null`
- **THEN** 服务不修改 cookies 表中该域的数据；响应 `cookies` 字段为 `0`

#### Scenario: 同时携带 cookies 与 local_storage
- **WHEN** 客户端在同一个 `/push` 请求里同时提供 `cookies` 数组和 `local_storage` 数组
- **THEN** 两者在同一个 SQLite 事务内完成镜像替换；任一写入失败则整体回滚，DB 保持推送前状态

#### Scenario: 请求体格式非法
- **WHEN** 客户端 POST 的 JSON 缺少 `domain` 字段或字段类型不匹配
- **THEN** 服务返回 HTTP 400，响应体包含错误描述，DB 不发生任何写入

### Requirement: SQLite 持久化与共享访问
系统 SHALL 在 Tauri `app_data_dir()` 下创建 `data.db`，使用 WAL 日志模式（`PRAGMA journal_mode=WAL`），schema 包含 `domains`、`cookies` 与 `local_storage` 三张表。`cookies` 与 `local_storage` 通过 `domain_id INTEGER REFERENCES domains(id)` 外键关联。文件 MUST 允许独立的外部进程以只读方式打开。

#### Scenario: 首次启动建表
- **WHEN** dev-tools 第一次启动，数据目录中不存在 `data.db`
- **THEN** cookie_bridge 初始化时创建 `data.db`，建立 `domains(id PRIMARY KEY AUTOINCREMENT, domain_name TEXT NOT NULL UNIQUE, urls TEXT, description TEXT, created_at DATETIME, updated_at DATETIME)`、`cookies(domain_id, name, path, value, expires, secure, http_only, updated_at, PRIMARY KEY(domain_id, name, path))` 与 `local_storage(domain_id, key, value, updated_at, PRIMARY KEY(domain_id, key))` 三张表，并启用 WAL 模式

#### Scenario: 从旧 schema 迁移
- **WHEN** 应用启动检测到旧表 `cookies` / `local_storage` 含 `domain TEXT` 列
- **THEN** 系统 DROP 旧 `cookies` / `local_storage` 表，重建新 schema（含 `domains` 与新的 `cookies` / `local_storage`）；**旧 cookies / local_storage 历史数据直接丢弃，不做迁移**

#### Scenario: 外部进程并发只读
- **WHEN** dev-tools 正在运行且 `data.db` 中已有数据；另一个 Go 进程以只读模式打开同一文件并执行 `SELECT * FROM cookies`
- **THEN** Go 进程能读到 dev-tools 已提交的数据，不会因为锁冲突报 `database is locked`

### Requirement: 前端页面查询接口
系统 SHALL 提供 Tauri command 供前端查询当前持久化的 cookie 数据：`cookie_bridge_list_domains` 返回所有出现过的域名列表（按更新时间倒序），`cookie_bridge_get_domain` 接受 `domain` 参数返回该域当前所有 cookie 与 localStorage 条目。`cookie_bridge_list_domains` 返回结构由字符串数组改为包含 `id` 与 `domainName` 的对象数组。

#### Scenario: 列出所有域名
- **WHEN** 前端调用 `invoke('cookie_bridge_list_domains')`
- **THEN** 返回对象数组，每个元素为 `{ "id": 1, "domainName": "example.com", "urls": [...], "description": "...", "createdAt": "...", "updatedAt": "..." }`，按各域 `MAX(updated_at)` 倒序排列

#### Scenario: 查询某域详情
- **WHEN** 前端调用 `invoke('cookie_bridge_get_domain', { domain: 'example.com' })`
- **THEN** 返回结构 `{ "domain": { "id": 1, "domainName": "example.com", ... }, "cookies": [...], "localStorage": [...] }`

#### Scenario: 查询不存在的域
- **WHEN** 前端查询一个 DB 中没有的域名
- **THEN** 返回 `{ "domain": null, "cookies": [], "localStorage": [] }`，不抛错

### Requirement: 写入完成后的实时事件通知
（无修改，保持原 spec 不变）

### Requirement: 前端 cookie-bridge 页面
（无修改，保持原 spec 不变）

### Requirement: 下游查询端点（HTTP）
HTTP 服务 SHALL 提供两个 GET 端点供内部 Go 应用查询已持久化的 cookie / localStorage 数据。`GET /domains` 返回含 `id` 的对象数组；`GET /domains/:domain` 返回含 `domain` 元信息的对象。

#### Scenario: 列出所有域名
- **WHEN** Go 应用向 `http://127.0.0.1:8765/domains` 发起 GET 请求
- **THEN** 服务返回 HTTP 200，响应体为 JSON 对象数组，每个元素含 `id`、`domainName`、`urls`、`description`、`createdAt`、`updatedAt`，按各域 `MAX(updated_at)` 倒序排列

#### Scenario: 查询某域详情
- **WHEN** Go 应用向 `http://127.0.0.1:8765/domains/example.com` 发起 GET 请求
- **THEN** 服务返回 HTTP 200，响应体：
  ```json
  {
    "domain": {
      "id": 1,
      "domainName": "example.com",
      "urls": ["https://example.com"],
      "description": "",
      "createdAt": "2026-05-07 10:00:00",
      "updatedAt": "2026-05-07 10:00:00"
    },
    "cookies": [
      {
        "domainName": "example.com",
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
        "domainName": "example.com",
        "key": "user_id",
        "value": "42",
        "updatedAt": 1700000000000
      }
    ]
  }
  ```

#### Scenario: 查询不存在的域名
- **WHEN** Go 应用查询一个 DB 中不存在的域名
- **THEN** 服务返回 HTTP 200，响应体为 `{"domain":null,"cookies":[],"localStorage":[]}`，不抛错

#### Scenario: DB 查询失败
- **WHEN** `/domains` 或 `/domains/:domain` 在查询 SQLite 时发生错误
- **THEN** 服务返回 HTTP 500，响应体为 `{"error":"db error: ..."}`

### Requirement: 跨源访问支持
（无修改，保持原 spec 不变）
