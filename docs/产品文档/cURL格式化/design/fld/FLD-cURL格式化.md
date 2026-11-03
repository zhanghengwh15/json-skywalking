---
title: cURL 格式化前端设计
status: completed
business: dev-tools
module: curl-formatter
requirement: cURL格式化
created: 2026-11-03
updated: 2026-11-03
author: AI 辅助编写
related-code:
  - repo: tauri-app
    path: src/views/CurlFormatter.vue
  - repo: tauri-app
    path: src/utils/curlFormatter.ts
  - repo: tauri-app
    path: src/router/index.ts
  - repo: tauri-app
    path: src/App.vue
---

# cURL 格式化前端设计

关联 [PRD](../../prd/PRD-cURL格式化.md)。本功能为本地纯前端功能，无新增 IPC、HTTP、CLI、持久化、原生权限或数据库升级。

## 页面与状态

- `src/App.vue` 在 HTTP 解析之后增加 `terminal` 图标菜单，`src/router/index.ts` 懒加载 `/curl-formatter`。
- `src/views/CurlFormatter.vue` 使用现有暗色主题、按钮和颜色变量；双栏输入 / 输出，小屏改为纵向排列。
- 顶部包含版本目标、正文参数选择、排版开关和 JSON 补头开关；底部展示具体转换项和不能自动判断的事项。
- 页面内 `ref` 管理输入、选项、结果、错误、剪贴板反馈；无跨页面共享状态或自动保存。
- 输入变化清除结果与错误，避免复制过期命令；选项变化重新转换。空输入禁用转换，错误状态不显示旧输出。
- 用户点击转换或按 Ctrl / ⌘ + Enter 生成结果；示例直接填入并转换，清空回到空态。
- 单行 / 多行仅改变参数之间的续行排版，不改动正文内部的真实换行。
- 复制 / 粘贴使用 `navigator.clipboard`；运行时不支持或权限失败时反馈手动复制 / 粘贴方式。反馈定时器在卸载时清理。

## 转换模型与流程

`src/utils/curlFormatter.ts` 导出纯函数：

```typescript
formatCurl(source: string, options?: Partial<CurlFormatOptions>): CurlFormatResult
```

输入包含正文参数 `-d | --data-binary`、多行开关与 JSON 补头开关。返回命令、推断方法、URL、请求头数量、内联正文 UTF-8 字节数、转换项和提示；无法转换时抛出可展示的 `Error`。

1. 清理外层 Markdown 代码围栏及 `$ curl` 提示符。
2. 按状态读取 Shell 单词，处理普通引号、转义、ANSI-C 字面量及续行。变量与命令替换不执行，遇到时拒绝。
3. 按 cURL 7.29 常用选项集合解析参数，识别短参数组合与 `--option=value`；未知项报错。
4. 恢复纯 URL；剔除命令尾部明确的中文操作说明并记录。
5. 识别正文与文件引用；`--json` 内联片段直接拼接，保留已有请求头，按配置补齐 JSON 头。
6. JSON 仅用于判断合法性，不将解析结果重新写回；输出使用原文字面值，避免数字精度变化。
7. 以 POSIX 单引号重新引用每个值，内含单引号通过拼接表达，生成单行或反斜杠续行命令。

`--data-raw` 正文以 `@` 开头、无法离线合并的正文组合、新版未知选项和无法表达的字节均明确拒绝，不以“兼容”为由改变语义。已有文件正文选项保留其换行读取行为。`--http1.1` 转换为旧版默认行为并显示说明。

## 数据与运行环境

- 输入和输出只保留在组件内存，不写 localStorage、数据库、日志或历史。
- 用户命令不传给 Shell / eval / 网络；纯文本通过 Vue 插值和 textarea 展示。
- 不新增依赖。Node 测试使用已安装的 TypeScript 编译工具，Shell 测试替换为仅捕获参数的临时 `curl`，不能发送网络请求。
- 浏览器可验证本功能；剪贴板能力仍需分别考虑运行时权限。Tauri WebView 和用户旧版 Linux 的实际验证记录在测试文档。

## 验证

运行 `npm run test:curl`、`npm run build`，页面验证菜单、示例、切换、编辑失效、错误、复制与快捷键。覆盖关系见[测试文档](../../testcase/testcase-兼容转换-2026-11-03.md)。
