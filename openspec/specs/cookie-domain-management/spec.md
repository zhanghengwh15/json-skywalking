# cookie-domain-management Specification

## Purpose
TBD - created by archiving change add-domain-management. Update Purpose after archive.

## Requirements
### Requirement: Domain 实体与数据库模型
系统 SHALL 将 `domain` 提升为一等实体，建立 `domains` 表，包含 `id`（自增主键）、`domain_name`（唯一，NOT NULL）、`urls`（JSON 数组字符串，可选）、`description`（文本，可选）、`created_at` 与 `updated_at`（SQLite datetime）。`cookies` 与 `local_storage` 两张表 SHALL 由 `domain TEXT` 改为 `domain_id INTEGER NOT NULL REFERENCES domains(id)` 外键关联。

#### Scenario: 首次启动建表与迁移
- **WHEN** dev-tools 启动，`Db::open` 发现旧表 `cookies` / `local_storage` 含 `domain TEXT` 列（即 `domain_id` 列不存在）
- **THEN** 系统在单事务内执行：创建 `domains` 表；用旧表 `DISTINCT domain` 回填 `domains.domain_name`；创建带 `domain_id` 的新 `cookies_new` / `local_storage_new` 并回填数据；DROP 旧表；RENAME 新表；创建兼容视图 `v_cookies` / `v_local_storage`

#### Scenario: 数据结构保持 camelCase
- **WHEN** 任意 Rust struct（`Domain`、`CookieItem`、`LocalStorageItem`）被序列化为 JSON
- **THEN** 字段名使用 camelCase，例如 `domainName`、`createdAt`、`updatedAt`、`httpOnly`

### Requirement: /push 推送时自动维护 Domain 行
系统 SHALL 在 `/push` 处理中，写入 `cookies` / `local_storage` 之前，先根据 `payload.domain` 执行 `INSERT OR IGNORE INTO domains(domain_name) VALUES(?)`，获取对应 `domain_id`，再以该 `domain_id` 写入子表。

#### Scenario: 推送新域
- **WHEN** Chrome 扩展 POST `/push` 携带 `{"domain":"newsite.com",...}`，且 `newsite.com` 在 `domains` 中不存在
- **THEN** 系统先在 `domains` 插入 `newsite.com` 行，再以其 `id` 写入 cookies / local_storage

#### Scenario: 推送已有域
- **WHEN** Chrome 扩展 POST `/push` 携带已知域名
- **THEN** 系统复用已有 `domain_id`，不重复插入 domain 行，仅做 cookies / local_storage 的镜像替换

### Requirement: Domain CRUD HTTP 端点
系统 SHALL 在 `127.0.0.1:8765` 上提供 `GET /api/domains`、`POST /api/domains`、`GET /api/domains/:id`、`PUT /api/domains/:id`、`DELETE /api/domains/:id` 五个端点，返回统一格式 `{ "success": true, "data": ... }` 或 `{ "success": false, "message": "..." }`。

#### Scenario: 列表查询
- **WHEN** Go 应用 GET `/api/domains`
- **THEN** 返回所有 `domains` 行数组，每个元素为 `{ "id": 1, "domainName": "...", "urls": [...], "description": "...", "createdAt": "...", "updatedAt": "..." }`

#### Scenario: 创建域
- **WHEN** Go 应用 POST `/api/domains` 携带 JSON body `{"domainName": "example.com", "urls": ["https://example.com/login"], "description": ""}`
- **THEN** 插入新行，返回 HTTP 200，data 为创建后的完整对象（含生成的 `id`）

#### Scenario: 创建时 domainName 已存在
- **WHEN** POST 的 `domainName` 已存在于表中
- **THEN** 返回 HTTP 409，`{ "success": false, "message": "domain already exists" }`

#### Scenario: 更新域
- **WHEN** Go 应用 PUT `/api/domains/1` 携带 `{"urls": ["https://new.example.com"], "description": "updated"}`
- **THEN** 仅更新提供的字段，`updated_at` 刷新，返回更新后的完整对象

#### Scenario: 删除域
- **WHEN** Go 应用 DELETE `/api/domains/1`
- **THEN** 级联删除该 `domain_id` 在 `cookies` 与 `local_storage` 中的关联行，再删除 `domains` 行；返回 `{ "success": true, "data": { "deleted": true } }`

### Requirement: URL 匹配域端点
系统 SHALL 提供 `GET /api/domains/match?url=<url>`，从 URL 解析 host，先精确匹配 `domains.domain_name`，若未命中则遍历 `domains.urls` JSON 数组做后缀匹配，返回命中的域记录与其 cookies / localStorage 快照。

#### Scenario: 精确 host 匹配
- **WHEN** GET `/api/domains/match?url=https://example.com/page`
- **THEN** 解析 host 为 `example.com`，命中 `domain_name = 'example.com'`，返回 `{ "domain": { ... }, "cookies": [...], "localStorage": [...] }`

#### Scenario: urls 字段后缀匹配
- **WHEN** GET `/api/domains/match?url=https://sub.example.com/page` 且 `example.com` 的 `urls` 含 `"https://*.example.com"`
- **THEN** 精确 host 未命中，回退到 urls 匹配，命中后返回对应域及其快照

#### Scenario: 未匹配
- **WHEN** GET `/api/domains/match?url=https://unknown.com`
- **THEN** 返回 HTTP 200，data 为 `null`（或 `{ "domain": null, "cookies": [], "localStorage": [] }`）

### Requirement: Domain CRUD Tauri Commands
系统 SHALL 提供 6 个 Tauri command，供前端调用：
- `domain_list`：返回所有 domains 数组
- `domain_create(body: CreateDomain)`：创建并返回完整对象
- `domain_get(id: i64)`：返回单条或错误
- `domain_update(id: i64, body: UpdateDomain)`：更新并返回完整对象
- `domain_delete(id: i64)`：删除并返回确认
- `domain_match_url(url: String)`：按 URL 匹配并返回快照

#### Scenario: 前端列出所有域
- **WHEN** 前端调用 `invoke('domain_list')`
- **THEN** 返回 `Domain[]`，与 HTTP GET `/api/domains` 数据一致

#### Scenario: 前端创建域
- **WHEN** 前端调用 `invoke('domain_create', { domainName: 'test.com', urls: ['https://test.com'] })`
- **THEN** 返回创建的 `Domain` 对象

### Requirement: Domain CRUD CLI 子命令
系统 SHALL 支持 `dev-tools-cli domain <action>` 子命令（Windows 控制台二进制；macOS DMG 仅装 GUI 二进制 `dev-tools`，命令名换成 `dev-tools` 即可），直接读写 `data.db`：
- `list`：列出所有域
- `create --domain-name <n> [--urls <json-array>] [--description <d>]`：创建
- `get <id>`：查单条
- `update <id> [--domain-name <n>] [--urls <json>] [--description <d>]`：更新
- `delete <id>`：级联删除
- `match --url <url>`：URL 匹配

#### Scenario: CLI 列出域
- **WHEN** 终端执行 `dev-tools-cli domain list`（macOS：`dev-tools domain list`）
- **THEN** 打印格式化 JSON 数组到 stdout

#### Scenario: CLI 创建域
- **WHEN** 终端执行 `dev-tools-cli domain create --domain-name example.com --urls '["https://example.com"]'`（macOS：`dev-tools domain create ...`）
- **THEN** 打印创建的域对象 JSON

### Requirement: 下游 Domain 列表与详情端点
系统 SHALL 修改 `GET /domains` 与 `GET /domains/:domain`（原 cookie_bridge 查询端点），使其返回含 `id` 的域对象而非纯字符串。

#### Scenario: 查询域名列表
- **WHEN** Go 应用 GET `/domains`
- **THEN** 返回 `[{ "id": 1, "domainName": "example.com", "urls": [...], ... }, ...]`，按 `updated_at` 倒序

#### Scenario: 查询某域快照
- **WHEN** Go 应用 GET `/domains/example.com`
- **THEN** 返回 `{ "domain": { "id": 1, "domainName": "example.com", ... }, "cookies": [...], "localStorage": [...] }`
