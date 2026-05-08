## Why

≈≈≈≈

## What Changes

- **BREAKING**: 新增 `domains` 表（id 主键、`domainName` 唯一、可冗余 `urls`/`description` 等字段），`cookies` / `local_storage` 由 `domain` 字段改为 `domain_id` 外键关联；`/push` 入口在写入前自动 upsert `domains` 行
- 新增 HTTP 端点 `GET /api/domains`、`POST /api/domains`、`GET /api/domains/:id`、`PUT /api/domains/:id`、`DELETE /api/domains/:id` 提供完整 CRUD，挂载在已有的 `127.0.0.1:8765` 上
- 新增按 URL 匹配域的端点 `GET /api/domains/match?url=<url>`：先从 URL 解析 host，再回退到 `urls` 字段做后缀/前缀匹配，命中则返回域记录与其 cookies / localStorage 快照
- 新增 Tauri command：`domain_list` / `domain_create` / `domain_get` / `domain_update` / `domain_delete` / `domain_match_url`，供前端使用
- 新增 CLI 子命令 `dev-tools domain <list|create|get|update|delete|match>`，与 `task-branch-group` 风格一致，输出格式化 JSON
- 修改 `cookie_bridge_list_domains` / `cookie_bridge_get_domain` / `GET /domains` / `GET /domains/:domain` 的返回结构：附带 `id` 字段，并保持向后兼容的 `domainName` / `urls` 元信息
- 对外 JSON 命名统一采用 **camelCase**（与现有 `task_branch_group` 风格一致）；URL 查询参数使用 snake_case
- 新增对接文档 `docs/domain-management-对接文档.md`，覆盖 HTTP / CLI / Tauri 三种调用方式、字段语义与匹配规则

## Capabilities

### New Capabilities
- `cookie-domain-management`: 把"域名"提升为带 id 的一等实体，提供 HTTP / CLI / Tauri 三套 CRUD 与 URL 匹配能力，并把 `cookies` / `local_storage` 改为通过 `domain_id` 外键关联

### Modified Capabilities
- `cookie-bridge`: `cookies` / `local_storage` schema 由 `domain TEXT` 改为 `domain_id INTEGER REFERENCES domains(id)`；`/push` 在写入前 upsert `domains` 行；`/domains` 与 `cookie_bridge_list_domains` 返回结构由字符串数组改为含 `id` 的对象数组

## Impact

- **Schema 迁移**：首次启动检测到旧 `cookies` / `local_storage` 表（含 `domain TEXT` 列）时，自动建 `domains` 表、按 `DISTINCT domain` 回填、为旧表新增 `domain_id` 列并填充、删除旧 `domain` 列；迁移使用单事务，失败回滚
- **Rust 模块新增**：`src-tauri/src/cookie_bridge/domain.rs`（实体 + DAO）、`http.rs` 路由扩展、`commands.rs` 命令扩展
- **Rust main.rs 变更**：新增 `Domain { action: DomainAction }` 子命令分支与对应的 `run_domain_cli`
- **前端**：现有 `/cookie-bridge` 页面的左侧域名列表项渲染由字符串改为读取 `domainName` 字段；类型定义同步调整。新增可选的「域管理」入口（不在本次变更必须实现，但 spec 留口）
- **跨进程契约变更**：Go 程序原本读取 `cookies.domain` 列须改为通过 `domain_id` JOIN `domains.domain_name`；本次提供向后兼容视图 `v_cookies` / `v_local_storage`（含 `domain` 列）以减少外部改动成本
- **网络监听**：仍然只占用 `127.0.0.1:8765`，无新增端口
- **不影响**：`task_branch_group` 模块、剪贴板、托盘、JSON / SQL / HTTP Parser
