## Why

开发者需要临时接收任意测试参数并返回固定数据，且能核对请求是否到达。现有 Cookie Bridge 端口和解析历史不能提供可配置的测试服务。

## What Changes

- 新增测试服务页面，配置本机端口、多个精确 URL 路径、方法、状态码和固定 JSON 响应，支持启动与停止。
- 新增请求日志页面，展示请求与响应详情，支持筛选、刷新和清空。
- 根据配置预览并导出 Markdown 调用说明与 curl 示例。

## Capabilities

### New Capabilities

- `http-test-server`: 可配置本地测试 HTTP 服务、请求日志和文档导出。

### Modified Capabilities

无。

## Impact

- 新增 Vue 页面、导航、路由与 Rust `http_test_server` 模块及 IPC。
- 复用 Axum、Tokio、Serde 和现有依赖；现有 8765 服务、CLI 和 SQLite 契约不变。
- 新增应用数据目录内的配置文件；请求日志仅保存在当前进程内。
- 对接说明见后续 `docs/http-test-server-对接文档.md`，需求入口为 `docs/产品文档/HTTP测试服务/MOC.md`。
