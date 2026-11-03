---
paths:
  - "src/**/*.vue"
  - "src/**/*.ts"
  - "src/**/*.css"
  - "vite.config.ts"
---
# Vue 与 TypeScript 规则

- 使用 Vue 3 Composition API 与 TypeScript，沿用相邻组件格式；不批量迁移已有页面。
- 共享 DTO、props 与公开工具函数定义明确类型；外部错误与输入优先使用 `unknown` 并收窄，避免新增 `any` 或绕过类型检查。
- `invoke` 使用实际注册的 command 名、参数名和返回结构；`Option`、空值及数字精度依契约处理，不从 Rust 命名推断 JSON。
- 页面内状态优先用 `ref` / `reactive`；实际需要跨页面共享时再用已安装的 Pinia，不为单页强制增加 store。
- 新页面同步核对 `src/router/index.ts` 与 `src/App.vue` 导航，样式沿用当前工具页设计。
- 处理 loading、空数据、错误与重复提交；异步结束后恢复交互，不能仅在控制台记录用户可见失败。
- 事件监听、轮询和快捷键注册有对应清理；Tauri 调用需考虑浏览器预览没有原生运行时。
- 保留既有中文界面，不自动加入 React、Axios、Zod、UI 库、权限框架或国际化依赖。
- 前端修改执行 `npm run build`；桌面交互另在 Tauri 验证。设计文档按 `docs-prd.md` 输出 FLD。
