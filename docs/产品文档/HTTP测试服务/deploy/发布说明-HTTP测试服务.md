---
title: HTTP 测试服务发布与验证说明
status: completed
business: dev-tools
module: http-test-server
requirement: HTTP测试服务
created: 2026-11-03
updated: 2026-11-06
author: AI 辅助编写
related-code:
  - repo: tauri-app
    path: src-tauri/src/http_test_server/
  - repo: tauri-app
    path: src/views/HttpTestServer.vue
  - repo: tauri-app
    path: src/views/HttpTestLogs.vue
---

# HTTP 测试服务发布与验证说明

## 范围与构建

本次新增桌面测试服务、请求日志和 Markdown 导出。需求和实现设计见 [MOC](../MOC.md)，完整 HTTP / IPC / CLI 差异见[对接文档](../../../http-test-server-对接文档.md)。

2026-11-06 增量：每个接口可完整修改响应体和 Content-Type，默认仍为原有 JSON 示例；允许 JSON、文本、HTML、XML 和空内容，最多 64 KiB，原文返回。新增接口 cURL 生成、预览、复制和页面快捷键 `⌘ / Ctrl + Alt + C`，统一方法下拉框、操作按钮和 MD 折叠入口样式。

沿用 npm 和 Cargo 锁文件。版本字段未修改：package.json 为 0.1.0，Cargo.toml 为 1.0.0；Tauri 配置没有单独覆盖版本。本次未构建或发布安装包。

开发启动：仓库根目录运行 `npm run tauri dev`。发布构建：`npm run tauri build`。macOS / Windows 安装步骤沿用 [CLI 安装与 PATH 说明](../../../dev-tools-cli-安装与PATH.md)，本次没有新安装钩子、PATH 或命令行行为。GUI / 控制台的既有 stdout 差异保持原样。

## 配置、权限与升级

- 测试服务默认端口 8766，可配置，仅监听 127.0.0.1；既有 8765 服务不变。启动失败需检查占用；不扩大为局域网监听。
- 成功启动后保存 Tauri 应用数据目录内的 `http-test-server.json`，包含端口与接口配置；重启应用只读取配置，不自动启动服务。
- 本次新增可缺省的 `endpoints[].contentType`，旧配置读取时自动补齐 `application/json; charset=utf-8`；下次成功启动写回。没有单独迁移脚本，配置损坏仍提示并回退默认值。响应正文不再强制 JSON；服务运行时编辑锁定，停止、修改并重启后生效。
- 请求日志保存在内存中，最近 200 条，不增加文件日志或数据库表；无需 SQLite 升级、迁移 SQL 或停写操作。
- 新 command 已注册在 `lib.rs`，由应用管理状态提供服务；没有新增 Tauri 插件、capability 权限或外部文件路径输入。
- 导出使用 Tauri 下载目录解析与 Rust 文件写入，最多 4 MiB，创建唯一文件名；下载目录不可写时页面显示错误。
- 依赖 `http-body-util` 由已锁定的传递依赖改为显式直接依赖，用于区分正文超限错误；Tokio 显式启用 net、time、io-util，未升级依赖版本。
- 若需保留自定义测试配置，退出应用后备份上述配置文件即可。既有业务数据库与本变更无关，按既有流程管理。

## 安装后检查

1. 打开「测试服务」，以默认配置启动，调用[默认示例](HTTP测试服务-调用示例.md)，确认固定返回。
2. 修改请求入参和格式，确认返回一致；修改 URL 或方法检查 404；浏览器调用检查 OPTIONS 和跨域头。
3. 打开「请求日志」，核对详情、筛选、刷新、清空；停止服务后日志保留，退出应用后清除。
4. 导出 MD，核对当前配置、curl 示例及系统下载目录中的文件。修改配置需停止再启动。
5. 停止服务后确认端口可重新绑定；应用重启后确认配置恢复但服务处于停止状态；原 8765 服务仍按原契约工作。
6. 将不同接口分别设为文本、HTML、空正文及各自 Content-Type，启动后核对实际响应与日志；重启应用核对配置恢复。JSON 示例中的 code、message、data 可删除或替换。
7. 点击指定接口「生成 cURL」，核对命令方法和地址；聚焦另一个接口，按快捷键核对切换；点击复制或手动选取命令。展开 MD 入口，核对自定义正文及类型。HEAD / 204 / 205 / 304 仍无正文。

上述为验收操作说明，未执行部分不能据此认定通过。

## 实际验证记录

日期：2026-11-03；平台：macOS 15.7.7 / Apple Silicon，Rust 1.87.0。

| 验证 | 结果 | 覆盖与限制 |
| --- | --- | --- |
| `npm run build` | 通过 | Vue TypeScript 检查与 Vite 构建 |
| `cargo check --locked`，在 src-tauri 执行 | 通过 | 桌面、IPC 注册和 HTTP 模块编译 |
| `cargo test --locked`，在 src-tauri 执行 | 通过 | 5 个 Rust 测试，真实本机 TCP / HTTP，使用临时配置和虚构数据 |
| `node --test tests/httpTestServer.test.mjs` | 通过 | 4 个 MD 测试：当前配置内容、POSIX URL 引号、Markdown 围栏及非法配置 |
| `openspec validate add-http-test-server --strict` | 通过 | 规格和任务格式 |
| ego-browser 页面检查 | 通过 | 端口 / URL / 固定 JSON 编辑、添加接口、MD 预览更新、非法 JSON 提示、日志页导航与空态 |
| ego-browser + 模拟 IPC 交互 | 通过 | 启停状态、运行中禁止修改、导出内容与反馈、日志详情、方法及 URL 筛选、清空后同步移除详情；不代表原生 IPC 或文件写入实测 |
| Tauri 原生交互及文件导出 | 未执行 | 未启动新的完整桌面应用，也未写入用户下载目录；浏览器预览不能证明原生能力通过 |
| Windows 安装与运行 | 未执行 | 未做 Windows 实机、安装包或跨平台验收 |
| 生产安装包与用户业务验收 | 未执行 | 本次没有发布或安装操作 |

Rust 测试覆盖任意 JSON / 表单 / 文本 / 空请求体、固定响应、查询串记录、请求头凭据隐藏、404、ANY、OPTIONS、HEAD、204、1 MiB 超限、64 KiB 截断、200 条保留、端口占用、重复启动、停止释放端口、配置重载和不自动启动。未执行 15 秒超时、连接中断、强制停止的异常路径实测。

首次从仓库根目录执行 Cargo 时，本机全局 USTC 镜像拒绝连接，离线缓存也不足；改为在 `src-tauri` 目录执行后，项目 `.cargo/config.toml` 生效并完成检查。未改写全局 Cargo 配置。浏览器工具提示可用更新，未升级。

### 2026-11-06 增量验证

| 验证 | 结果 | 覆盖与限制 |
| --- | --- | --- |
| `npm run build` | 通过 | TypeScript 检查与 Vite 构建；新增响应类型、命令预览与快捷键 |
| `node --test tests/httpTestServer.test.mjs` | 通过，6 项 | 任意 / 空响应、默认兼容、Content-Type、UTF-8 大小上限、MD 围栏、POSIX URL 转义、指定接口 / ANY / HEAD 的 cURL 及配置错误 |
| `cargo check --locked` 与 `cargo test --locked`，在 src-tauri 执行 | 通过，7 项 Rust 测试 | 临时配置与真实本机 TCP；多接口独立正文、文本空白换行、JSON / 文本 / HTML / XML / 空正文、非 JSON 原文、类型、保存重载后实际返回、旧配置缺省兼容、非法头 / 超长正文，以及原有生命周期、请求限制和日志回归 |
| `openspec validate add-http-test-server --strict` | 通过 | 增量规格与任务格式 |
| ego-browser 页面交互 | 通过 | 非 JSON 响应编辑、指定第二个接口的快捷键生成 HEAD cURL、命令实际复制成功、模拟剪贴板失败时提示手动复制、Enter / Space 展开折叠、页面样式截图检查；多接口下文档卡片保持可见，折叠区不再被压缩 |
| Tauri 原生界面、Windows 与安装包 | 未执行 | 本轮未做原生窗口启停、系统下载目录导出或 Windows 实机验证，未构建发布安装包 |

Cargo 仍显示既有 `unused manifest key: build` 警告，不影响本次检查和测试通过。

## 回退

停止测试服务、退出应用后安装旧版本。旧版本不会读取新增配置文件；可保留该文件以便重新安装新版时恢复。无业务数据迁移或回滚 SQL。已经导出的 MD 为普通文档，回退不会自动删除。桌面关闭窗口只是隐藏到托盘，回退前应实际退出进程。

若回退到已有测试服务但仅接受合法 JSON 的版本：先备份 `http-test-server.json`，或在当前版本将正文恢复为合法 JSON 后成功启动保存，再停止退出。旧版本会忽略新增 Content-Type 字段并返回 JSON 类型；非 JSON 或空正文会触发旧版本的配置校验失败并回退默认值。重新升级后可恢复备份配置。
