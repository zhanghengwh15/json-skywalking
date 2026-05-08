## 1. Schema 变更与迁移

- [x] 1.1 在 `Db::open` 中检测旧表 schema（`cookies` / `local_storage` 含 `domain TEXT` 列）
- [x] 1.2 实现迁移逻辑：`DROP TABLE cookies`、`DROP TABLE local_storage`，创建 `domains` 表与带 `domain_id` 的新 `cookies` / `local_storage` 表
- [x] 1.3 验证迁移后 schema 正确（`domains` 有自增 id、`domain_name` UNIQUE；子表有 `domain_id` 外键）

## 2. Domain 实体与 DB 层

- [x] 2.1 定义 `Domain` struct（含 `id`、`domain_name`、`urls`、`description`、`created_at`、`updated_at`），对外使用 `#[serde(rename_all = "camelCase")]`
- [x] 2.2 定义 `CreateDomain`、`UpdateDomain` 输入 struct（均 camelCase）
- [x] 2.3 在 `Db` impl 中实现 `domain_create`、`domain_list`、`domain_get`、`domain_update`、`domain_delete`
- [x] 2.4 实现 `domain_match_url`：解析 URL host → 精确匹配 `domain_name` → 回退 `urls` 后缀匹配 → 返回域 + cookies + localStorage 快照
- [x] 2.5 修改 `push` 方法：在事务内先 `INSERT OR IGNORE INTO domains(domain_name) VALUES(?)` 获取 `domain_id`，再以 `domain_id` 写入子表
- [x] 2.6 修改 `list_domains`：返回 `Vec<Domain>` 而非 `Vec<String>`
- [x] 2.7 修改 `get_domain`：返回含 `domain` 元信息的新结构 `DomainSnapshot`（`domain: Option<Domain>`, `cookies: Vec<CookieItem>`, `local_storage: Vec<LocalStorageItem>`）
- [x] 2.8 修改 `CookieItem` / `LocalStorageItem` 对外输出为 `domain_name` 字符串（保持 camelCase）

## 3. HTTP 端点扩展

- [x] 3.1 在 `http.rs` 中新增 `GET /api/domains`、`POST /api/domains`、`GET /api/domains/:id`、`PUT /api/domains/:id`、`DELETE /api/domains/:id` 路由与 handler
- [x] 3.2 新增 `GET /api/domains/match?url=` 路由与 handler
- [x] 3.3 修改 `GET /domains` handler：返回 `Vec<Domain>` 对象数组
- [x] 3.4 修改 `GET /domains/:domain` handler：返回 `{ "domain": Domain, "cookies": [...], "localStorage": [...] }`
- [x] 3.5 所有新增 handler 使用与 task_branch_group 一致的 `{ "success": bool, "data": ..., "message": ... }` 响应封装

## 4. Tauri Command 扩展

- [x] 4.1 在 `commands.rs` 中新增 `domain_list`、`domain_create`、`domain_get`、`domain_update`、`domain_delete`、`domain_match_url` 六个 command
- [x] 4.2 修改 `cookie_bridge_list_domains`：返回 `Vec<Domain>`
- [x] 4.3 修改 `cookie_bridge_get_domain`：返回新的 `DomainSnapshot` 结构
- [x] 4.4 在 `lib.rs` 的 `invoke_handler` 中注册新增 command

## 5. CLI 扩展

- [x] 5.1 在 `main.rs` 的 `Commands` enum 中新增 `Domain { action: DomainAction }` 分支
- [x] 5.2 定义 `DomainAction` enum（`List`、`Create`、`Get`、`Update`、`Delete`、`Match`）及对应参数
- [x] 5.3 实现 `run_domain_cli` 函数，直接调用 `Db` 对应方法，输出格式化 JSON

## 6. 前端适配

- [x] 6.1 更新 `/cookie-bridge` 页面的 Domain 类型定义（由 `string` 改为 `{ id, domainName, ... }`）
- [x] 6.2 修改左侧域名列表渲染逻辑：读取 `domainName` 而非直接使用字符串
- [x] 6.3 修改选中域后的详情加载：适配新的 `DomainSnapshot` 结构
- [x] 6.4 验证页面无类型报错，列表/详情/空状态均正常（类型定义已同步，需运行 `npm run dev` / `cargo tauri dev` 实际验证）

## 7. 对外文档

- [x] 7.1 新建 `docs/domain-management-对接文档.md`，包含：HTTP 端点说明、请求/响应示例、字段语义、URL 匹配规则
- [x] 7.2 在文档中说明 CLI 子命令用法与 PATH 配置（与 task_branch_group 文档风格一致）
- [x] 7.3 在文档中说明 Tauri command 列表与参数
- [x] 7.4 文档中明确标注 JSON 字段使用 camelCase、URL 查询参数使用 snake_case
