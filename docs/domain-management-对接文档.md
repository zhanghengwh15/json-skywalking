# 域名管理（domain-management）对接说明

Domain 是 `cookie_bridge` 中 cookie / localStorage 数据的归属实体。应用启动后会在本机 **8765** 端口提供 HTTP；命令行子命令直接读写同一套数据库（需与桌面端使用相同的应用数据目录，见下文）。

Domain 表包含 `id`、`domainName`、`urls`（JSON 数组，用于 URL 匹配）、`description` 等字段。`cookies` 与 `local_storage` 表通过 `domain_id` 外键关联到 `domains`。

---

## 一、给人看的（简明版）

### HTTP（推荐在应用已打开时使用）

- 地址前缀：`http://127.0.0.1:8765/api/domains`
- **查列表**：`GET` 根路径，返回所有域记录。
- **新建**：`POST`，JSON 里写 `domainName`（必填）、`urls`（JSON 数组字符串，可选）、`description`（可选）。
- **查一条 / 改 / 删**：`GET` `PUT` `DELETE` 路径里带数字 id，例如 `/api/domains/12`。
- **URL 匹配**：`GET /api/domains/match?url=https://example.com/page`，返回命中的域及其 cookies / localStorage。
- 成功时一般是 `{ "success": true, "data": ... }`；失败时 `{ "success": false, "message": "..." }`。

### 命令行（适合脚本、不依赖 GUI）

可执行文件名是 **`dev-tools`**。**DMG / Windows 安装包**只会把应用装进「应用程序」或「Program Files」，**不会在终端里注册 `dev-tools` 命令**；从源码编译时二进制在 `src-tauri/target/...`，同样默认不在 PATH。直接敲 `dev-tools` 会 `command not found` 是正常现象。

**已用安装包的用户**：用 `.app` 里 `Contents/MacOS/dev-tools` 的全路径，或给该路径做 symlink / alias；**Windows** 用安装目录里的 `dev-tools.exe` 全路径或把该目录加入用户 Path。详见：[`dev-tools-cli-安装与PATH.md`](dev-tools-cli-安装与PATH.md) 第 **0** 节。

仅开发本机有仓库时，可不配置 PATH，在仓库根执行：

`cargo run --manifest-path src-tauri/Cargo.toml -- domain list`

子命令统一为：`dev-tools domain <子命令> ...`（无子命令时会启动桌面应用，脚本里务必带子命令）。

| 作用   | 示例 |
|--------|------|
| 列表   | `dev-tools domain list` |
| 新建   | `dev-tools domain create --domain-name example.com --urls '["https://example.com"]'` |
| 单条   | `dev-tools domain get 12` |
| 更新   | `dev-tools domain update 12 --description "updated"`（至少改一个字段） |
| 删除   | `dev-tools domain delete 12`（级联删除关联的 cookies / localStorage） |
| 匹配   | `dev-tools domain match --url https://example.com/page` |

输出为格式化 JSON，打印到标准输出。

### 数据文件在哪

与 Tauri 应用标识 `com.tauri.devtools` 一致，例如 macOS：`~/Library/Application Support/com.tauri.devtools/data.db`。
HTTP 与桌面端共用该库；CLI 默认也打开该路径下的库。

---

## 二、给 AI / 自动集成的说明（详细版）

本章节约定：除非特别说明，**请求/响应 JSON 字段采用 camelCase**（与 Serde `rename_all = "camelCase"` 一致）。**URL 查询参数**使用 **snake_case** 字段名（`url`）。

### 1. 运行环境与存储

| 项目 | 说明 |
|------|------|
| HTTP 监听 | `127.0.0.1:8765`（与 cookie_bridge 同进程） |
| 路由挂载 | Axum `nest("/api/domains", ...)`，故资源根为 `/api/domains` |
| 数据库 | `app_data_dir/data.db`，表 `domains`、`cookies`、`local_storage` |
| CORS | `permissive` |
| 请求体上限 | 整站路由层约 10MB（含本 API） |

删除接口为**物理级联删除**：删除 `domains` 行时，外键 `ON DELETE CASCADE` 会自动删除 `cookies` 与 `local_storage` 中关联的行。

### 2. 数据模型

**实体 `Domain`（列表/详情返回）**

| JSON 字段 | 类型 | 说明 |
|-----------|------|------|
| `id` | number | 自增主键 |
| `domainName` | string | 域名，唯一（如 `example.com`） |
| `urls` | string \| null | JSON 数组字符串，存储关联 URL 模板，用于 URL 匹配 |
| `description` | string \| null | 描述 |
| `createdAt` | string | 创建时间（SQLite datetime） |
| `updatedAt` | string | 更新时间 |

**创建体 `CreateDomain`（仅 POST body）**

| JSON 字段 | 类型 | 必填 | 说明 |
|-----------|------|------|------|
| `domainName` | string | 是 | 域名 |
| `urls` | string \| null | 否 | JSON 数组字符串，如 `["https://example.com"]` |
| `description` | string \| null | 否 | 描述 |

**更新体 `UpdateDomain`（仅 PUT body）**

| JSON 字段 | 类型 | 说明 |
|-----------|------|------|
| `domainName` | string \| null | 不提供则不修改 |
| `urls` | string \| null | 不提供则不修改 |
| `description` | string \| null | 不提供则不修改 |

### 3. HTTP 端点详情

#### 3.1 列表查询

```
GET /api/domains
```

**响应 200**
```json
{
  "success": true,
  "data": [
    {
      "id": 1,
      "domainName": "example.com",
      "urls": "[\"https://example.com\",\"https://*.example.com\"]",
      "description": null,
      "createdAt": "2026-05-07 10:00:00",
      "updatedAt": "2026-05-07 10:00:00"
    }
  ],
  "message": null
}
```

#### 3.2 创建域

```
POST /api/domains
Content-Type: application/json

{
  "domainName": "newsite.com",
  "urls": "[\"https://newsite.com\"]",
  "description": "测试站点"
}
```

**响应 200（成功）**
```json
{
  "success": true,
  "data": {
    "id": 2,
    "domainName": "newsite.com",
    "urls": "[\"https://newsite.com\"]",
    "description": "测试站点",
    "createdAt": "2026-05-07 10:05:00",
    "updatedAt": "2026-05-07 10:05:00"
  },
  "message": null
}
```

**响应 409（domainName 已存在）**
```json
{
  "success": false,
  "data": null,
  "message": "domain already exists"
}
```

#### 3.3 查询单条

```
GET /api/domains/2
```

**响应 200**
```json
{
  "success": true,
  "data": {
    "id": 2,
    "domainName": "newsite.com",
    ...
  },
  "message": null
}
```

**响应 404**
```json
{
  "success": false,
  "data": null,
  "message": "not found"
}
```

#### 3.4 更新域

```
PUT /api/domains/2
Content-Type: application/json

{
  "description": "更新后的描述"
}
```

**响应 200**
返回更新后的完整对象。`updated_at` 自动刷新。

#### 3.5 删除域

```
DELETE /api/domains/2
```

**响应 200**
```json
{
  "success": true,
  "data": { "deleted": true },
  "message": null
}
```

级联删除该 `domain_id` 关联的所有 `cookies` 与 `local_storage` 行。

#### 3.6 URL 匹配

```
GET /api/domains/match?url=https://sub.example.com/page
```

匹配逻辑：
1. 从 URL 解析 host（`sub.example.com`）
2. 先精确匹配 `domains.domain_name`
3. 若未命中，遍历各 domain 的 `urls` JSON 数组，做后缀匹配（支持 `*.example.com` 通配前缀）

**响应 200（命中）**
```json
{
  "success": true,
  "data": {
    "domain": {
      "id": 1,
      "domainName": "example.com",
      ...
    },
    "cookies": [...],
    "localStorage": [...]
  },
  "message": null
}
```

**响应 200（未命中）**
```json
{
  "success": true,
  "data": {
    "domain": null,
    "cookies": [],
    "localStorage": []
  },
  "message": null
}
```

### 4. Tauri Command 列表

前端通过 `@tauri-apps/api/core` 的 `invoke` 调用：

| Command | 参数 | 返回 |
|---------|------|------|
| `domain_list` | 无 | `Domain[]` |
| `domain_create` | `{ domainName, urls?, description? }` | `Domain` |
| `domain_get` | `{ id: number }` | `Domain` |
| `domain_update` | `{ id: number, domainName?, urls?, description? }` | `Domain` |
| `domain_delete` | `{ id: number }` | `boolean` |
| `domain_match_url` | `{ url: string }` | `{ domain?: Domain, cookies: CookieItem[], localStorage: LocalStorageItem[] }` |

### 5. `/push` 推送端点（接口不变，内部行为升级）

Chrome 扩展推送 cookie / localStorage 的 `/push` 端点，**外部请求/响应格式保持不变**，但内部写入逻辑已升级为：

1. 先根据 `payload.domain` 在 `domains` 表中执行 `INSERT OR IGNORE`
2. 获取该 domain 的 `id`，再以 `domain_id` 写入 `cookies` / `local_storage`

这意味着推送一个全新域名时，系统会自动在 `domains` 表中创建对应记录（`urls` / `description` 为空）。

**请求**

```
POST /push
Content-Type: application/json

{
  "domain": "example.com",
  "cookies": [
    {
      "domain": "example.com",
      "name": "sid",
      "value": "abc123",
      "path": "/",
      "expires": 1700000000,
      "secure": 0,
      "httpOnly": 0
    }
  ],
  "local_storage": [
    { "key": "user_id", "value": "42" }
  ],
  "ts": 1700000000000
}
```

**响应 200**

```json
{
  "ok": true,
  "cookies": 1,
  "local_storage": 1
}
```

**响应 400（缺少 domain）**

```json
{
  "error": "domain is required"
}
```

### 6. Cookie Bridge 查询端点变更

`GET /domains` 与 `GET /domains/:domain`（原 cookie_bridge 查询端点）的返回结构已升级：

- `GET /domains` 返回 `Domain[]` 对象数组（不再是字符串数组）
- `GET /domains/:domain` 返回 `{ "domain": Domain, "cookies": [...], "localStorage": [...] }`

`CookieItem` 与 `LocalStorageItem` 的 `domain` 字段已更名为 `domainName`（camelCase）。
