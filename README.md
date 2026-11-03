# Tauri App

Dev Tools 是基于 Tauri 的本地桌面开发工具箱，包含以下六个功能页面，并提供本地 HTTP 与 CLI 对接能力。

## 功能页面

### 1. JSON 解析
- 智能 JSON 数据解析与格式化
- 支持剪贴板粘贴、历史记录、结构树、右键菜单、快捷键

### 2. SQL 解析
- SQL 语句格式化与高亮
- 支持多种数据库语法

### 3. HTTP 解析
- HTTP 请求内容解析与格式化
- 支持历史记录、请求预览、快捷操作

### 4. 时间戳转换
- 时间戳与日期时间转换

### 5. Cookie Bridge
- 接收浏览器扩展推送的 Cookie / localStorage
- 域名管理、关联数据查看与本地接口查询

### 6. 任务分支管理
- 管理 TB 任务与 Git 分支的关联关系
- 支持页面操作、本地 HTTP 与 CLI 调用

## 技术栈
- 前端：Vue 3 + TypeScript + Vite
- 桌面端：Tauri 2.x + Rust
- 状态管理：Pinia
- 路由：Vue Router
- 本地存储：tauri-plugin-store + SQLite（rusqlite）
- 本地服务与 CLI：Axum + Clap
- 全局快捷键：tauri-plugin-global-shortcut

## 启动与构建

```bash
npm install
npm run tauri dev
```

```bash
npm run tauri build
```

---

如有问题请提交 issue。

## 项目协作与文档

- 项目结构、开发命令与 AI 协作入口：[AGENTS.md](AGENTS.md)。
- Cookie Bridge、域名、任务分支和 CLI 对接说明：[文档导航](docs/README.md)。
- 新需求的 PRD、设计、发布和测试文档规则：[docs-prd.md](.codex/rules/common/docs-prd.md)。
