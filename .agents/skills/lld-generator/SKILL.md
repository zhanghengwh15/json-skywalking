---
name: lld-generator
description: 为 Dev Tools 编写或更新 Rust、Tauri、Axum、SQLite 与 CLI 的详细设计 LLD，核实共享逻辑、契约、事务和数据升级。用于后端或整体技术设计文档任务。
---

# Dev Tools 详细设计

先读 `AGENTS.md`、`.codex/rules/common/docs-prd.md` 和 `.codex/rules/rust/coding-style.md`，定位 PRD、规格及相关模块。

## 核实来源

- `src-tauri/src/lib.rs`：初始化、插件、command 注册。
- 相关模块的 `commands.rs`、`http.rs`、`db.rs`：入口适配、字段、错误、查询与写入。
- `src-tauri/src/cli.rs` 和两个二进制入口：子命令、输出、应用数据目录及平台差异。
- `src-tauri/tauri.conf.json`、capability、Cargo 配置与当前对接文档，仅在受影响时读取。

代码结构通过可用 CodeGraph 定位并集中读取；文档与代码冲突时记录来源，已有行为按实现核实。

## 输出

写入 `docs/产品文档/{需求}/design/lld/LLD-{需求}.md`，按实际影响覆盖：

1. 目标、范围、PRD 和功能编号。
2. 架构、模块职责、入口到共享逻辑及数据流。
3. IPC / HTTP / CLI 契约与错误语义；字段细节引用统一对接文档。
4. 数据模型、SQLite 表 / 索引 / 外键、事务边界与已有数据升级。
5. 输入校验、幂等、软删除 / 级联删除、并发、锁和异步生命周期。
6. 平台与运行时依赖，日志和敏感数据处理。
7. 受影响文件、实施拆分、验收与验证方式。
8. 发布影响、回退限制和待确认事项。

已实现与拟新增方案分开表述，拟新增字段不能伪装成现有 DTO。没有数据库或某类入口影响时简述理由，不补空章节。

涉及升级、权限、配置、安装包或发布顺序时，使用项目 `deployment-doc-generator` 衔接发布文档，将最终脚本和顺序回填摘要。更新 MOC；当前任务只要求设计时不执行代码或数据变更。
