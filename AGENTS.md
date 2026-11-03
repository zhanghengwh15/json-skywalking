# Dev Tools 项目协作说明

## 项目介绍

Dev Tools 是面向开发者的本地桌面工具箱，使用 Vue 3 + TypeScript + Vite 构建界面，使用 Tauri 2 + Rust 提供桌面能力。已有 JSON 解析、SQL 解析、HTTP 解析、时间戳转换、Cookie Bridge、任务分支管理六个页面。Cookie Bridge 内含域名管理，接收浏览器扩展推送的 Cookie / localStorage，并提供本地查询与持久化能力。

- 前端入口：`src/main.ts`；应用布局：`src/App.vue`；路由：`src/router/index.ts`。
- Rust 库入口与 command 注册：`src-tauri/src/lib.rs`。
- GUI 入口：`src-tauri/src/main.rs`；控制台入口：`src-tauri/src/bin/dev-tools-cli.rs`；共用子命令：`src-tauri/src/cli.rs`。
- 本地服务：Axum，当前绑定 `127.0.0.1:8765`；包括 `/health`、`/push`、`/domains`、`/api/domains`、`/api/task-branch-groups`。
- 数据存储：rusqlite / SQLite，应用数据目录中的 `data.db`，开启 WAL 和外键；解析历史另由 `desktop/store.rs` 管理。
- 主要平台：macOS、Windows。Windows 的 GUI 与控制台二进制存在 stdout 行为差异，发布与脚本说明需区分。

## 目录导航

| 路径 | 用途 |
| --- | --- |
| `src/views/` | 六个工具页面 |
| `src/utils/` | 前端工具函数 |
| `src/assets/styles/` | 页面样式 |
| `src-tauri/src/desktop/` | 剪贴板、历史存储、托盘与快捷键 |
| `src-tauri/src/cookie_bridge/` | 域名、Cookie、localStorage 的数据库、HTTP 与 IPC |
| `src-tauri/src/task_branch_group/` | 任务分支关联的数据库、HTTP 与 IPC |
| `src-tauri/capabilities/` | Tauri 能力权限 |
| `src-tauri/tauri.conf.json` | 窗口、构建与安装包配置 |
| `docs/` | 对接、安装说明与需求文档；入口见 `docs/README.md` |
| `openspec/` | 变更提案、规格与归档；保留既有工作流 |
| `.agents/skills/` | 项目技能的唯一维护位置，包含既有 OpenSpec 技能 |
| `.codex/rules/` | 按任务读取的 Markdown 规则，由本文件显式引用 |

## 按任务读取规则

`.codex/rules/**/*.md` 是协作说明，并非命令执行权限配置；不能假设工具会根据 `paths` 自动加载。开始相关任务时，主动读取下表中的规则。

| 任务 | 规则 |
| --- | --- |
| 所有代码增量修改 | [.codex/rules/common/coding-style.md](.codex/rules/common/coding-style.md) |
| 创建、更新、评审、整理 PRD / 设计 / 测试 / 发布文档 | [.codex/rules/common/docs-prd.md](.codex/rules/common/docs-prd.md) |
| Vue、TypeScript、前端样式 | [.codex/rules/typescript/coding-style.md](.codex/rules/typescript/coding-style.md) |
| Rust、Tauri、Axum、SQLite、CLI | [.codex/rules/rust/coding-style.md](.codex/rules/rust/coding-style.md) |
| 验证代码变更或记录测试结果 | [.codex/rules/common/testing.md](.codex/rules/common/testing.md) |

## 常用命令

在仓库根目录执行，优先复用 `package-lock.json` 与 `Cargo.lock`，不随任务切换包管理器。

```bash
npm ci
npm run dev
npm run tauri dev
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

`npm run dev` 仅启动前端；IPC、托盘、剪贴板、全局快捷键等能力需在 Tauri 中验证。`npm run build` 包含 `vue-tsc --noEmit` 与 Vite 构建。当前 `package.json` 未配置 lint、test 或独立 typecheck 脚本，不要报告不存在的命令已通过。Rust 工具链以实际依赖、锁文件和本机工具链为准，不以旧的 `rust-version` 字段推断依赖兼容性。

## 实施约定

- 默认用中文沟通与编写文档，保留代码标识和固定技术术语。
- 先理解用户目标与现有实现，保持修改聚焦；不顺带重排代码、升级依赖或修改生成目录 `dist/`、`target/`、`node_modules/`。
- 新增页面同步检查路由和应用导航；新增 IPC 同步检查 `lib.rs` 的 `generate_handler!` 注册和所需能力权限。
- 改动共享能力时按适用范围核对 Vue IPC、HTTP、CLI 三种入口；不能从其中一种入口的字段或错误格式推断另外两种。
- JSON 字段、查询参数、IPC 参数的命名以实际 Serde / Axum / Tauri 定义为准。已有 `/push` 包含历史兼容格式，不统一重命名。
- 数据库变更说明已有数据如何升级。任务分支使用软删除，域名删除会级联删除关联 Cookie / localStorage，不能混用。
- Cookie、token、localStorage 和剪贴板可能含敏感数据；文档及验证使用虚构样例，诊断日志不扩散真实值。
- 现有对接文档保留原路径；涉及契约变更时同步对应说明。新需求按文档规则归档，不批量搬迁旧文档。
- 简单修复直接实施；跨模块功能或用户要求提案时使用已有 OpenSpec 技能。当前任务授权不因引入模板而增加审批步骤。
- 交付说明改动、验证结果和未覆盖的限制；文档完成、构建通过、业务验收通过是不同状态。

## 项目技能

需要相关产物时先读取对应 `SKILL.md`，根据当前任务范围使用，不为小修复强制生成整套文档。

| 技能 | 用途 |
| --- | --- |
| [prd-writer](.agents/skills/prd-writer/SKILL.md) | 需求、业务规则与验收标准 |
| [fld-generator](.agents/skills/fld-generator/SKILL.md) | Vue 页面、状态与桌面交互设计 |
| [lld-generator](.agents/skills/lld-generator/SKILL.md) | Rust / SQLite / HTTP / IPC / CLI 详细设计 |
| [deployment-doc-generator](.agents/skills/deployment-doc-generator/SKILL.md) | 桌面发布、数据库升级与回退说明 |
| [ai-test-case-generation](.agents/skills/ai-test-case-generation/SKILL.md) | 从需求生成测试用例与覆盖关系 |
| [api-sync](.agents/skills/api-sync/SKILL.md) | 核实并更新 IPC、HTTP、CLI 对接文档 |
| `openspec-propose` / `openspec-apply-change` / `openspec-archive-change` / `openspec-explore` | 使用 `.agents/skills/` 下既有同名技能 |

<!-- CODEGRAPH_START -->
## CodeGraph

结构性问题优先使用 CodeGraph：符号定义用 `codegraph_search`，调用方 / 被调用方用 `codegraph_callers` / `codegraph_callees`，流程用 `codegraph_trace`，影响面用 `codegraph_impact`，签名与源码用 `codegraph_node`，任务上下文用 `codegraph_context`，批量源码用 `codegraph_explore`，文件列表用 `codegraph_files`，索引健康用 `codegraph_status`。

- 架构问题先 `codegraph_context`，再用一次 `codegraph_explore` 读取关键符号；流程问题先 `codegraph_trace`，再用一次 `codegraph_explore`。
- 直接回答，不派子代理重复探索，不用 grep 重验 AST 结果，不循环调用 `codegraph_node` 读取大量符号。
- 字符串、注释、日志等字面查询用 `rg`；已定位文件可直接读取。工具不可用时明确说明，再用原生文件搜索与读取完成必要调查。
- 文件监听约延迟 500ms；编辑后不立即查询索引。
- 若 `.codegraph/` 不存在且工具返回未初始化，询问用户：“这个项目还没有初始化 CodeGraph，需要运行 `codegraph init -i` 构建索引吗？”
<!-- CODEGRAPH_END -->
