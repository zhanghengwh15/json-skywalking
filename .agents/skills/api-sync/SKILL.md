---
name: api-sync
description: 核实 Dev Tools 的 Rust DTO、Tauri IPC、Axum HTTP 与 CLI，生成或更新对应对接文档并报告差异。用于接口契约同步，不凭文档任务改写业务实现。
---

# Dev Tools 契约同步

先读 `AGENTS.md` 和 `.codex/rules/common/docs-prd.md`。确定目标模块与实际对外入口，保留现有对接文档路径。

## 定位来源

- Tauri：相关 `commands.rs` 与 `lib.rs` 中的注册，核实参数映射、返回类型和错误。
- HTTP：相关 `http.rs` 的挂载路径、方法、查询 / 请求体、响应包装和状态码。
- DTO / 数据：相关 `db.rs` 的 Serde 注解、模型、校验和持久化语义。
- CLI：`cli.rs` 中参数、默认值、子命令、输出与退出行为；核对入口和数据目录。

结构问题优先 CodeGraph，批量读取相关符号，不假设只读一种入口即可得到完整契约。

## 更新契约

- Cookie Bridge 更新 `docs/cookie-bridge-对接文档.md`。
- 域名管理更新 `docs/domain-management-对接文档.md`。
- 任务分支更新 `docs/task-branch-group-对接文档.md`。
- CLI 安装 / PATH 变化更新 `docs/dev-tools-cli-安装与PATH.md`。
- 新模块确需对接文档时新建 `docs/{模块}-对接文档.md` 并登记 `docs/README.md`，当前需求 MOC 链接同一文件。

文档分别列 HTTP、IPC、CLI 的入口及差异；字段表包含名称、类型、可空 / 必填、单位与默认值；示例必须符合实际契约。

重点核对：

- JSON camelCase、query snake_case、command 参数名不可互相推断。
- `/push` 的历史别名、bool / 整数兼容、未提供与空值的行为。
- 任务分支软删除、域名级联删除以及共享数据库路径。
- HTTP envelope、IPC 的 `Result`、CLI 输出格式与实际错误状态。

提供“代码事实 / 原文档 / 本次修正”的差异摘要。不将设计提案写成现有接口，不因发现漂移擅自改实现；产品目标冲突记为待确认。示例使用虚构登录态，更新需求关联与文档日期。
