# MOC - HTTP 测试服务

## 文档列表

| 类型 | 文档 | 状态 |
| --- | --- | --- |
| 接口与使用 | [HTTP 测试服务对接](../../http-test-server-对接文档.md) | completed |
| 默认调用示例 | [应用生成的 MD 示例](deploy/HTTP测试服务-调用示例.md) | completed |
| 发布与验证 | [发布说明](deploy/发布说明-HTTP测试服务.md) | completed |

## 关联信息

- 用户需求：桌面内配置端口、URL、固定返回；任意测试入参；生成 MD 使用说明；请求进入日志页面。
- 模块：`http_test_server`。
- 代码：`src-tauri/src/http_test_server/`、`src/views/HttpTestServer.vue`、`src/views/HttpTestLogs.vue`、`src/utils/httpTestServer.ts`。
- OpenSpec：[提案](../../../openspec/changes/add-http-test-server/proposal.md)、[设计](../../../openspec/changes/add-http-test-server/design.md)、[规格](../../../openspec/changes/add-http-test-server/specs/http-test-server/spec.md)、[任务](../../../openspec/changes/add-http-test-server/tasks.md)。
- 文档完成、代码检查及自动测试通过；真实 Tauri 窗口操作、下载目录写入和 Windows 实机验收未执行，详见发布说明。

## 变更日志

- 2026-11-03 实现测试服务、会话请求日志和文档导出，登记契约、示例及验证结果。
