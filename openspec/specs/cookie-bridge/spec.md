# cookie-bridge Specification

## Purpose
TBD - created by archiving change add-cookie-bridge. Update Purpose after archive.
## Requirements
### Requirement: 本地 HTTP 推送端点
系统 SHALL 在 dev-tools 启动时尝试在 `127.0.0.1:8765` 启动一个本地 HTTP 服务，提供 `POST /push` 与 `GET /health` 两个端点。监听地址 MUST 限定为 `127.0.0.1`，不得监听其他网络接口。

#### Scenario: 健康检查
- **WHEN** 任意进程对 `http://127.0.0.1:8765/health` 发起 GET 请求
- **THEN** 服务返回 HTTP 200，响应体为字符串 `ok`

#### Scenario: 端口被占用时优雅降级
- **WHEN** dev-tools 启动时发现 `127.0.0.1:8765` 已被其他进程占用，`TcpListener::bind` 失败
- **THEN** 系统记录错误日志，发送一条托盘通知告知用户 cookie 桥不可用，并继续完成 dev-tools 主进程启动；其他功能（剪贴板、解析器、托盘）必须仍然可用

#### Scenario: 仅允许本机访问
- **WHEN** 同一局域网内其他主机尝试访问 `http://<dev-tools-host-ip>:8765/health`
- **THEN** 连接被操作系统层拒绝（因为只 bind 在 127.0.0.1）

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
系统 SHALL 在每次 `/push` 写入事务成功提交后，通过 Tauri 全局事件 `cookie-bridge:updated` 广播被更新的域名（payload 为字符串 domain）。前端页面 MUST 在打开时监听该事件并据此重新拉取数据。

#### Scenario: 写入触发事件
- **WHEN** `/push` 处理完成，事务已提交
- **THEN** Rust 端通过 `app_handle.emit("cookie-bridge:updated", "<domain>")` 广播事件

#### Scenario: 前端实时刷新
- **WHEN** 前端 `/cookie-bridge` 页面已打开并 `listen('cookie-bridge:updated')`，外部 push 触发了 `example.com` 的写入
- **THEN** 前端在收到事件后重新调用 `cookie_bridge_list_domains` 与（若当前选中域为 `example.com`）`cookie_bridge_get_domain`，UI 显示更新后的状态

#### Scenario: 写入失败不发事件
- **WHEN** `/push` 的事务因任何原因回滚
- **THEN** 系统 MUST NOT 发送 `cookie-bridge:updated` 事件

### Requirement: 前端 cookie-bridge 页面
系统 SHALL 在前端提供路由 `/cookie-bridge`，对应页面以"状态视角"展示当前 cookie 数据：左侧为按域名分组的列表，右侧为选中域的当前 cookie 表与 localStorage 表。

#### Scenario: 打开页面
- **WHEN** 用户导航到 `/cookie-bridge`
- **THEN** 页面调用 `cookie_bridge_list_domains` 渲染左侧列表；若列表非空，默认选中第一个域并加载其详情

#### Scenario: 切换域
- **WHEN** 用户点击左侧某个域名
- **THEN** 右侧表格切换为该域当前的 cookies / localStorage 数据

#### Scenario: 空状态提示
- **WHEN** 当前 DB 中没有任何域
- **THEN** 页面显示空状态提示文案，告知用户 cookie 桥已就绪、等待扩展推送

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
HTTP 服务 SHALL 配置 CORS 中间件，允许任意 Origin、任意常用方法（GET、POST、OPTIONS）、任意常用请求头，以确保 Chrome 扩展的 background script 能成功跨源调用。

#### Scenario: 扩展端预检请求
- **WHEN** Chrome 扩展发起 `OPTIONS /push` 预检请求
- **THEN** 服务返回 HTTP 204，包含 `Access-Control-Allow-Origin: *`、`Access-Control-Allow-Methods` 含 POST、`Access-Control-Allow-Headers` 含 `Content-Type`

#### Scenario: 扩展端实际推送
- **WHEN** 预检通过后扩展发起 `POST /push`
- **THEN** 浏览器不报 CORS 错误，请求按"cookie / localStorage 推送接收与镜像写入"中的规范处理

