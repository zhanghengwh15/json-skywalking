## Context

cookie_bridge 模块目前使用 `cookies` 和 `local_storage` 两张表，以 `domain TEXT` 为键直接存储。外部通过 `GET /domains` 只能获得扁平的域名字符串列表，没有元信息管理能力，也无法按 URL 反向匹配域。task_branch_group 已实现了一套 CRUD 模式（HTTP + Tauri command + CLI），可作为参考。

## Goals / Non-Goals

**Goals:**
- 将 "domain" 提升为带 `id` 的一等实体，建立 `domains` 表
- `cookies` / `local_storage` 通过 `domain_id` 外键关联
- 提供 HTTP / Tauri / CLI 三套一致的 CRUD 接口
- 提供 `GET /api/domains/match?url=...` 按 URL 匹配域
- 旧数据自动迁移，不丢失
- 对外 JSON 统一使用 camelCase
- 编写对接文档 `docs/domain-management-对接文档.md`

**Non-Goals:**
- 修改 Chrome 扩展的推送协议（`/push` 的 JSON body 保持不变，仍为 `{"domain":"...",...}`）
- 修改 cookie_bridge 前端页面视觉设计（仅数据结构变更）
- 修改除 domain 外的其他模块

## Decisions

### 1. 迁移策略：直接 DROP 旧表重建，不保留历史数据
旧 `cookies` / `local_storage` 数据可直接丢弃。方案为：
1. `DROP TABLE IF EXISTS cookies`
2. `DROP TABLE IF EXISTS local_storage`
3. 创建 `domains` 表
4. 创建带 `domain_id` 的新 `cookies` / `local_storage` 表

**替代方案**：创建新表并迁移旧数据。否决原因：产品决策明确历史数据无需保留，直接重建最简单。

### 2. 表结构：`urls` 存 JSON 数组字符串
`domains.urls` 存储 `["https://example.com", "https://*.example.com"]` 形式的 JSON 数组。匹配时先精确 host 匹配，再回退到 `urls` 数组中的后缀匹配。

**替代方案**：单独建 `domain_urls` 子表。否决原因：功能简单，单表 JSON 足够，避免过度设计。

### 3. 三套接口（HTTP / Tauri / CLI）共享同一套 DB 方法
与 task_branch_group 保持一致：所有业务逻辑在 `Db` impl 中，HTTP handler、Tauri command、CLI runner 均调用 `Db` 方法，不在层间重复逻辑。

### 4. 返回结构统一使用 `serde(rename_all = "camelCase")`
所有对外 Rust struct（`Domain`, `CookieItem`, `LocalStorageItem`）均使用 `#[serde(rename_all = "camelCase")]`，确保 JSON 字段为 camelCase。

### 5. `/push` 入口在写入前自动 upsert domain
当 Chrome 扩展推送 `{"domain":"example.com",...}` 时，Rust 端先 `INSERT OR REPLACE INTO domains(domain_name) VALUES(?)` 获取/创建 domain_id，再以该 id 写入 cookies / local_storage。避免外部推送端感知 domain 表的存在。

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|----------|
| 旧数据丢失导致用户困惑 | 产品决策已明确可丢弃；文档中说明 schema 变更后历史 cookies 不再保留 |
| `domain_id` 外键导致推送时 domain 行不存在 | `INSERT OR REPLACE` 确保 domain 行一定存在 |
| 前端旧代码仍引用 `domain` 字符串 | 前端类型定义同步改为 `domainName` / `id` 字段，本次变更同时修改前端 |
| CLI 二进制名 `dev-tools` 不在 PATH | 文档中明确说明如何找到二进制路径（与 task_branch_group 文档一致） |
