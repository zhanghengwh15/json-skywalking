## Why

当前开发工具箱缺少对 AI SQL 任务与 Git 分支关联关系的管理能力。需要建立一个"任务-分支-工程分组"关联表，支持在桌面端和命令行中对任务与前后端分支的映射关系进行增删改查，以支撑后续 AI SQL 生成任务的工程上下文追踪。

## What Changes

- 在 SQLite 数据库中新建 `task_branch_group` 表，存储任务 ID、分支名称、表名和分组类型（前端/后端）
- 对外暴露两层接口：
  - Tauri `invoke` 命令（供前端 Vue 页面调用）
  - Axum HTTP API（供外部脚本/Chrome 扩展调用，合并到现有 8765 端口）
- 新增 Vue 页面 `TaskBranchGroup`，提供完整的增删改 UI，并注册到侧边栏菜单
- 新增 CLI 子命令（基于 `clap`），支持不启动 GUI 直接操作数据：
  - `dev-tools.exe task-branch-group list|create|update|delete|get`
- `Cargo.toml` 新增 `clap` 依赖，`main.rs` 改造为 CLI/GUI 双模式入口

## Capabilities

### New Capabilities
- `task-branch-group-crud`: 任务-分支-工程分组关联数据的完整增删改查能力，包含数据库模型、Tauri 命令、HTTP API、前端页面和 CLI 命令

### Modified Capabilities
- （无现有 spec 需要修改）

## Impact

- **后端**: 新增 `src-tauri/src/task_branch_group/` 模块（4 个文件），修改 `lib.rs`、`main.rs`、`Cargo.toml`
- **前端**: 新增 `src/views/TaskBranchGroup.vue`，修改 `src/App.vue`、`src/router/index.ts`
- **HTTP API**: 在 `cookie_bridge::http` 中通过 `nest` 挂载新路由 `/api/task-branch-groups`
- **依赖**: 新增 `clap` crate
- **数据库**: 复用现有 `app_data_dir/data.db`，初始化时追加建表语句和触发器
