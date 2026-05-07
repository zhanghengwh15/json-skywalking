# 任务分支分组（task_branch_group）对接说明

数据表示「TB 名称 / 任务 ID / Git 分支 / 前后端分组」的关联，存于本地 SQLite（`data.db` 表 `task_branch_group`）。  
应用启动后会在本机 **8765** 端口提供 HTTP；命令行子命令直接读写同一套数据库（需与桌面端使用相同的应用数据目录，见下文）。

---

## 一、给人看的（简明版）

### HTTP（推荐在应用已打开时使用）

- 地址前缀：`http://127.0.0.1:8765/api/task-branch-groups`
- **查列表**：`GET` 根路径，可选查询参数 `keyword`、`task_id`、`branch_name`（均为可选，可组合）。
- **新建**：`POST`，JSON 里写 `tbName`、`taskId`、`branchName`、`groupType`（1=前端，2=后端）、`createBy`（数字，可填 0）。
- **查一条 / 改 / 删**：`GET` `PUT` `DELETE` 路径里带数字 id，例如 `/api/task-branch-groups/12`。
- 成功时一般是 `{ "success": true, "data": ... }`；失败时 `{ "success": false, "message": "..." }`。

### 命令行（适合脚本、不依赖 GUI）

可执行文件名是 **`dev-tools`**。**DMG / Windows 安装包**只会把应用装进「应用程序」或「Program Files」，**不会在终端里注册 `dev-tools` 命令**；从源码编译时二进制在 `src-tauri/target/...`，同样默认不在 PATH。直接敲 `dev-tools` 会 `command not found` 是正常现象。  

**已用安装包的用户**：用 `.app` 里 `Contents/MacOS/dev-tools` 的全路径，或给该路径做 symlink / alias；**Windows** 用安装目录里的 `dev-tools.exe` 全路径或把该目录加入用户 Path。详见：[`dev-tools-cli-安装与PATH.md`](dev-tools-cli-安装与PATH.md) 第 **0** 节。

仅开发本机有仓库时，可不配置 PATH，在仓库根执行：

`cargo run --manifest-path src-tauri/Cargo.toml -- task-branch-group list`

子命令统一为：`dev-tools task-branch-group <子命令> ...`（无子命令时会启动桌面应用，脚本里务必带子命令）。

| 作用   | 示例 |
|--------|------|
| 列表   | `dev-tools task-branch-group list` 可加 `--task-id xxx`、`--branch-name yyy` |
| 新建   | `dev-tools task-branch-group create --tb-name a --task-id b --branch-name c --group-type 1` |
| 单条   | `dev-tools task-branch-group get 12` |
| 更新   | `dev-tools task-branch-group update 12 --branch-name new`（至少改一个字段） |
| 删除   | `dev-tools task-branch-group delete 12`（软删除） |

输出为格式化 JSON，打印到标准输出。

### 数据文件在哪

与 Tauri 应用标识 `com.tauri.devtools` 一致，例如 macOS：`~/Library/Application Support/com.tauri.devtools/data.db`。  
HTTP 与桌面端共用该库；CLI 默认也打开该路径下的库。

---

## 二、给 AI / 自动集成的说明（详细版）

本章节约定：除非特别说明，**请求/响应 JSON 字段采用 camelCase**（与 Serde `rename_all = "camelCase"` 一致）。**URL 查询参数**在列表接口中使用 **snake_case** 字段名（`keyword`、`task_id`、`branch_name`）。

### 1. 运行环境与存储

| 项目 | 说明 |
|------|------|
| HTTP 监听 | `127.0.0.1:8765`（与 cookie_bridge 同进程，见 `cookie_bridge::init`） |
| 路由挂载 | Axum `nest("/api/task-branch-groups", ...)`，故资源根为 `/api/task-branch-groups` |
| 数据库 | `app_data_dir/data.db`，表 `task_branch_group` |
| CORS | `permissive` |
| 请求体上限 | 整站路由层约 10MB（含本 API） |

列表默认只返回 **`rec_status = 1`** 的有效记录。删除接口为**软删除**（将 `rec_status` 置 0），不是物理删行。

### 2. 数据模型

**实体 `TaskBranchGroup`（列表/详情返回）**

| JSON 字段 | 类型 | 说明 |
|-----------|------|------|
| `id` | number | 主键 |
| `tbName` | string | TB 名称 |
| `taskId` | string | 任务 ID |
| `branchName` | string | Git 分支名 |
| `groupType` | number | `1` 前端（web_engineer），`2` 后端（back_engineer） |
| `createTime` | string | 创建时间（SQLite datetime） |
| `modifyTime` | string | 修改时间 |
| `recStatus` | number | 1 有效，0 已删 |
| `createBy` | number | 创建人 ID |
| `modifyBy` | number | 修改人 ID |

**创建体 `CreateTaskBranchGroup`（仅 POST body）**

必填：`tbName`、`taskId`、`branchName`、`groupType`、`createBy`（整数，无业务约束时可传 `0`）。

**更新体 `UpdateTaskBranchGroup`（PUT body）**

全部为可选：`tbName`、`taskId`、`branchName`、`groupType`、`recStatus`、`modifyBy`。  
服务端若解析后**没有任何字段需要更新**，则视为未更新成功，HTTP 返回 **404**（`not found or no changes`）。至少包含一个要修改的字段。

### 3. HTTP 接口规格

**Base URL**：`http://127.0.0.1:8765/api/task-branch-groups`

#### 3.1 列表 `GET /api/task-branch-groups/`

- **Query**（均可省略，AND 组合）  
  - `keyword`：非空时，对 `tb_name`、`task_id` 做 `LIKE %keyword%`（两端模糊）。  
  - `task_id`：精确匹配 `task_id`。  
  - `branch_name`：精确匹配 `branch_name`。  
- 始终附加条件 `rec_status = 1`。  
- 排序：`id DESC`。  
- **成功 200**：`{ "success": true, "data": TaskBranchGroup[], "message": null }`  
- **失败**：`{ "success": false, "message": "..." }`，多为 500。

#### 3.2 创建 `POST /api/task-branch-groups/`

- **Header**：`Content-Type: application/json`  
- **Body**：`CreateTaskBranchGroup` JSON。  
- **成功 200**：`data` 为新建完整实体。  
- **失败**：4xx/5xx + `ErrorResponse`（同上 `success: false`）。

#### 3.3 单条查询 `GET /api/task-branch-groups/{id}`

- `id` 为路径参数整数。  
- 仅当 `rec_status = 1` 时视为存在。  
- **404**：记录不存在或已软删。

#### 3.4 更新 `PUT /api/task-branch-groups/{id}`

- **Body**：`UpdateTaskBranchGroup`（部分字段）。  
- 成功则 `data` 为更新后实体。  
- **404**：不存在、已删，或 body 无有效更新字段。

#### 3.5 删除 `DELETE /api/task-branch-groups/{id}`

- 软删除。  
- **成功 200**：`data` 为 `{"deleted": true}`（嵌在统一成功包装内，与代码一致：`success_response(HashMap::from([("deleted", true)]))` → 实际 JSON 在 `data` 对象内含 `deleted: true`）。  
- **404**：未找到或未删除成功。

**成功包装统一形状**（`task_branch_group::http::ApiResponse`）：

```json
{ "success": true, "data": <任意可序列化类型>, "message": null }
```

**错误包装**（非 200 场景中的 JSON 体之一）：

```json
{ "success": false, "message": "人类可读错误信息" }
```

集成方应对 `success === false` 或 HTTP 状态码做分支处理；不要仅依赖 HTTP 200 推断业务成功（创建路径在异常时也可能返回错误 JSON + 非 2xx，以实际响应为准）。

### 4. CLI 规格（`dev-tools`）

**入口**：`dev-tools task-branch-group <action>`（若未将 `dev-tools` 加入 PATH，请用 `cargo run --manifest-path src-tauri/Cargo.toml -- …` 或目标目录下的完整路径；见 [`dev-tools-cli-安装与PATH.md`](dev-tools-cli-安装与PATH.md)。）

与 HTTP 的差异：**CLI 的 `list` 不支持 `keyword`**，仅支持 `--task-id`、`--branch-name`（对应 DB 精确筛选）。需要关键词模糊搜请用 HTTP 或扩展 CLI。

各子命令行为与数据库一致；标准输出为 **pretty JSON**；更新失败或删除未命中时进程以 **退出码 1** 结束，并向 stderr 打印简短英文信息（`not found or no changes` / `not found`）。

**Create 参数**：`--tb-name`、`--task-id`、`--branch-name`、`--group-type`（i32）、`--create-by`（默认 `0`，i64）。

**Update 参数**：位置参数 `id`；可选 `--tb-name`、`--task-id`、`--branch-name`、`--group-type`、`--rec-status`、`--modify-by`。

### 5. 与其它入口的关系（避免混淆）

- **Tauri invoke**（前端 `invoke('task_branch_group_*')`）为另一套 IPC 接口，字段命名同样为 camelCase；本文件侧重 **HTTP + CLI**。  
- `task_branch_group_list` 的 Tauri 命令当前封装可能**未暴露** `task_id` / `branch_name` 筛选（以 `commands.rs` 为准）；HTTP 与 CLI `list` 支持更多筛选组合。

### 6. 集成检查清单（AI 自检用）

1. 桌面应用是否已启动且 8765 未被占用（否则 HTTP 不可用，日志见 `cookie-bridge.log`）。  
2. POST/PUT 是否使用 **camelCase** JSON；GET 列表查询是否使用 **snake_case** query。  
3. `groupType` 是否限制为业务约定的 `1` 或 `2`。  
4. 删除后是否仍用旧 `id` 调 GET（应 404）。  
5. 脚本自动化优先 **CLI + jq** 或 **curl + JSON**，并统一使用应用数据目录下的 `data.db`。

---

文档版本与实现对齐：`task_branch_group/http.rs`、`task_branch_group/db.rs`、`src-tauri/src/main.rs`（CLI）、`cookie_bridge/mod.rs`（端口与 DB 路径）。
