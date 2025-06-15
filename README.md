# Tauri App

本应用为桌面端开发工具，包含以下三个核心功能页面：

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

## 技术栈
- 前端：Vue 3 + TypeScript + Vite
- 桌面端：Tauri 2.x
- 状态管理：Pinia
- 路由：Vue Router
- 本地存储：tauri-plugin-store
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
