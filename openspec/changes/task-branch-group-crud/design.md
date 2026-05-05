## Context

项目为 Tauri + Vue 桌面应用，后端使用 Rust + SQLite (rusqlite) + Axum HTTP server，前端为 Vue 3 + Vue Router。已有 `cookie_bridge` 模块作为完整参考，覆盖了数据库操作、Tauri Commands、HTTP API、前端页面四层架构。本次需求是在同一架构下新增一个独立的 CRUD 模块。

## Goals / Non-Goals

**Goals:**
- 在 SQLite 中创建 `task_branch_group` 表并支持增删改查
- 暴露 Tauri invoke 命令供前端调用
- 暴露 HTTP API（复用现有 8765 端口）供外部调用
- 提供 Vue 页面支持可视化的增删改
- 提供 CLI 子命令支持无 GUI 操作

**Non-Goals:**
- 不涉及用户认证/权限校验（`create_by`、`modify_by` 仅保留字段，值为 0）
- 不引入 ORM 框架（继续使用 `rusqlite` 原生 SQL）
- 不修改 `cookie_bridge` 的现有功能逻辑

## Decisions

### 1. 复用现有 SQLite 数据库
- **选择**: 将 `task_branch_group` 表建到 `app_data_dir/data.db`
- **理由**: 减少数据库文件管理复杂度，与 `cookie_bridge` 共用 `Db` 实例
- **替代方案**: 独立 `aisql.db`（被否，增加管理成本）

### 2. HTTP API 合并到现有 Axum Router
- **选择**: 在 `cookie_bridge::http::create_router` 中通过 `.nest("/api/task-branch-groups", ...)` 挂载新路由
- **理由**: 复用端口绑定、CORS、body limit 等中间件，无需额外端口管理
- **替代方案**: 独立端口（被否，增加配置复杂度）

### 3. CLI 使用 `clap` 而非 `tauri-plugin-cli`
- **选择**: 在 `main.rs` 中引入 `clap` 解析参数，CLI 模式下直接操作数据库并退出，不启动 Tauri GUI
- **理由**: `tauri-plugin-cli` 主要用于向 Tauri 应用传参，不适合纯命令行工具场景；`clap` 生态成熟，支持子命令和自动生成 help
- **替代方案**: `tauri-plugin-cli`（被否，仍会尝试启动 GUI 事件循环）

### 4. `modify_time` 通过 SQLite 触发器维护
- **选择**: 创建 `AFTER UPDATE` 触发器自动更新 `modify_time`
- **理由**: SQLite 不支持 `ON UPDATE CURRENT_TIMESTAMP`；在应用层手动更新容易遗漏
- **替代方案**: 应用层每次 update 手动设置（被否，易遗漏且重复代码）

### 5. 前端页面风格与现有页面一致
- **选择**: 复用 `App.vue` 中定义的 CSS 变量和组件风格（glass-panel、btn、表格样式等）
- **理由**: 保持 UI 一致性，减少样式重复

## Risks / Trade-offs

| 风险 | 缓解措施 |
|------|----------|
| `main.rs` 引入 CLI 后 Windows 下可能仍弹出控制台窗口 | 保留 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`；CLI 模式编译为 debug 或单独 target 时使用 |
| Axum Router 合并后路由冲突 | 新路由统一加 `/api/task-branch-groups` 前缀，与现有 `/push`、`/domains` 隔离 |
| `clap` 增加二进制体积 | `clap` 开启 `derive` feature，编译优化后增量约 200KB，可接受 |

## Migration Plan

- 无数据迁移（新表），首次启动时 `Db::open` 自动建表
- 无配置迁移
- 回滚：删除 `task_branch_group` 表及相关文件即可

## Open Questions

- （无）
