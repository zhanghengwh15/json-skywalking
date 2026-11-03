---
name: fld-generator
description: 为 Dev Tools 的 Vue 3 页面编写或更新前端详细设计 FLD，核实路由、状态、桌面交互及 IPC 消费，关联 PRD 与后端契约。用于前端设计文档，不自动实施页面改动。
---

# Dev Tools 前端设计

先读 `AGENTS.md`、`.codex/rules/common/docs-prd.md` 和 `.codex/rules/typescript/coding-style.md`，定位当前 PRD、规格和已有设计。

## 核实范围

- `src/router/index.ts`、`src/App.vue`：页面入口、导航与布局。
- 当前需求涉及的 `src/views/`、`src/utils/`、样式和实际存在的共享状态。
- 页面使用的 `invoke` / event / 插件调用，对照 Rust command 注册、DTO、相关对接文档。
- 不假设仓库已有 `src/api/`、`src/stores/`、组件库或国际化模块；只引用真实路径，拟新增结构单独标记。

结构探索优先用可用的 CodeGraph，上下文集中读取相关符号，不重新扫描全仓库。

## 输出

写入 `docs/产品文档/{需求}/design/fld/FLD-{需求}.md`，按复杂度覆盖：

- PRD 与功能编号、现有行为和拟变更范围。
- 页面信息层级、组件边界、路由和导航变化。
- 本地 / 共享状态、派生状态、加载 / 空 / 错误 / 成功反馈。
- 操作流程、输入校验、重复提交、关闭与取消行为。
- IPC 参数和返回消费、事件订阅、轮询与释放时机；完整字段表链接到契约文档。
- 浏览器预览与真实 Tauri 的差异，剪贴板、快捷键或窗口能力按需说明。
- 关键验收与验证方式，关联 `FR-*` / `AC-*`。

需求尚未实现时，明确标注设计方案和待新增文件。不要把所有拟定方案写成代码事实。不因出设计文档而改页面代码。

更新 MOC 及文档元数据；后端契约缺失时说明所需字段和行为，按当前任务授权衔接 LLD 或契约文档，不能虚构已注册的 command。
