# HTTP 测试服务对接文档

版本：1.0；更新日期：2026-11-03；状态：已按代码核实。实现完成，实际验证与未测范围见[发布说明](产品文档/HTTP测试服务/deploy/发布说明-HTTP测试服务.md)。

## 使用方法

1. 打开 Dev Tools 桌面应用的「测试服务」。默认监听 `127.0.0.1:8766`，默认接口为 `ANY /test`，返回 HTTP 200。
2. 按需修改端口，添加多个 URL 路径，选择方法、状态码，填写固定 JSON 响应。
3. 点击「启动服务」，用 curl、Postman 或程序向页面展示的地址发送请求。输入参数无必填字段，不需要声明参数结构。
4. 打开「请求日志」查看接收时间、来源、请求头、正文、状态码和实际返回体；可按 URL、端口、状态码或方法筛选，点击 URL 查看详情。
5. 展开「MD 使用说明预览」，点击「导出 MD」，文档保存到系统下载目录，页面显示完整保存位置。文档按当前页面配置生成，包含 curl 示例。
6. 修改运行配置前先点击「停止服务」。关闭窗口会进入托盘，服务继续运行；停止服务或退出应用才结束监听。

默认请求示例，兼容 cURL 7.29 + sh / bash；Windows 可用 Git Bash 或 WSL：

```bash
curl -X POST --url 'http://127.0.0.1:8766/test?source=example' \
  -H 'Content-Type: application/json' \
  --data-binary '{"name":"测试数据","count":123,"extra":{"anything":true}}'
```

默认固定响应：

```json
{
  "code": 0,
  "message": "ok",
  "data": null
}
```

入参不会参与生成响应。表单、文本、任意合法 HTTP 请求头、任意查询参数和空请求体均可；JSON 请求体无需符合任何业务 schema，甚至无需是合法 JSON。

## HTTP 行为

| 项目 | 实际行为 |
| --- | --- |
| 地址 | 仅 `127.0.0.1`，HTTP，无认证，无 TLS |
| 端口 | 整数 1024–65535，默认 8766，排除已有服务使用的 8765 |
| URL | 以 `/` 开头的精确路径，最多 2048 个 ASCII 可见字符；不能以 `//` 开头或含空格、`?`、`#`，须能解析为 HTTP URI；中文用 URL 编码 |
| 方法 | 可配置 ANY、GET、POST、PUT、PATCH、DELETE、HEAD；ANY 接受任意方法，OPTIONS 总是自动预检 |
| 匹配 | 方法和路径精确匹配，查询参数不参与；末尾 `/` 有区别；不会把 `:id`、`*` 当作路由变量 |
| 接口数量 | 1–50 个；同方法同路径不重复，同路径 ANY 不能和其他方法重叠 |
| 返回 | 每个接口配置固定 JSON 字符串及整数状态码 200–599；JSON 必须有效，最大 64 KiB；响应不附加业务包装，保留配置的 JSON 原文 |
| 响应头 | `Content-Type: application/json; charset=utf-8` |
| 空响应 | HEAD、HTTP 204 / 205 / 304 不返回正文 |
| 404 | 路径或方法未匹配，返回 `{"error":"未配置该路径或请求方法"}` |
| OPTIONS | 任意路径返回 204，无正文，记录日志 |
| 413 | 请求体超过 1 MiB，返回 `{"error":"请求体不能超过 1 MiB"}` |
| 400 / 408 | 正文读取失败 / 超过 15 秒未读取完成，返回对应 `error` JSON |
| 跨域 | 允许所有 Origin；预检允许 GET、POST、PUT、PATCH、DELETE、HEAD、OPTIONS，并回显申请的头名称；不支持跨域凭据 |

应用重启后读取上次成功启动的配置，但不会自动监听。端口占用、配置非法、配置文件写入失败会使启动失败并在页面显示错误；重复启动被拒绝。停止重复调用可成功。请求体过大、读取失败或超时时，日志正文为空并标记不完整。

HTTP 协议解析在处理器前失败的连接不属于应用请求日志范围；正常进入处理器的成功请求、未匹配请求、OPTIONS 及正文错误均记录。

## Tauri IPC

以下 command 已在 `src-tauri/src/lib.rs` 注册。前端以 `invoke` 调用；成功直接返回表中数据，失败以字符串拒绝 Promise，没有 HTTP envelope。

| Command | 参数 | 返回 |
| --- | --- | --- |
| `http_test_server_status` | 无 | `ServerStatus` |
| `http_test_server_start` | `{ config: ServerConfig }` | `ServerStatus` |
| `http_test_server_stop` | 无 | `ServerStatus` |
| `http_test_server_logs` | 无 | `LogSummary[]`，最新在前，不传正文 |
| `http_test_server_log` | `{ id: number }`，正整数日志 ID | `RequestLog` 或 `null`，不存在或已淘汰时为 null |
| `http_test_server_clear_logs` | 无 | `null`，清空日志，ID 序列继续增长 |
| `http_test_server_export_markdown` | `{ content: string }`，不超过 4 MiB 的 UTF-8 文档 | 保存后的完整文件路径字符串 |

`ServerConfig` 字段全部必填、不允许 null：

| 字段 | 类型 | 默认 / 约束 |
| --- | --- | --- |
| `port` | number，Rust u16 | 8766，范围见 HTTP 行为 |
| `endpoints` | Endpoint[] | 默认一个 ANY /test |
| `endpoints[].method` | string | 默认 ANY |
| `endpoints[].path` | string | 默认 /test |
| `endpoints[].statusCode` | number，Rust u16 | 默认 200 |
| `endpoints[].responseBody` | string | 默认响应见上文；内容是 JSON 原文字符串，不能直接传对象 |

`ServerStatus`：`running: boolean`、`config: ServerConfig`、`lastError: string | null`。配置文件损坏时回退默认配置，并通过 `lastError` 提示；成功启动后清除错误。

`LogSummary` 字段：

| 字段 | 类型 / 空值 | 含义 |
| --- | --- | --- |
| `id` | number，不可空 | 会话内递增 ID，从 1 开始 |
| `timestamp` | string，不可空 | UTC RFC3339，毫秒精度；页面转换为本地时间 |
| `port` | number，不可空 | 接收请求的服务端口 |
| `method` | string，不可空 | 实际请求方法 |
| `url` | string，不可空 | 配置的本机地址 + 端口 + 请求 URI，含原始查询串 |
| `remoteAddress` | string，不可空 | 对端 IP 和端口 |
| `statusCode` | number，不可空 | 实际生成的 HTTP 状态码 |
| `durationMs` | number，不可空 | 处理器入口到构造响应的耗时，整数毫秒，不含网络传输完成时间 |
| `matchedPath` | string 或 null | 方法和路径匹配的配置路径；OPTIONS 即使匹配也只执行预检 |

`RequestLog` 在摘要字段外增加：`headers: [string, string][]`、`body: string`、`bodyTruncated: boolean`、`responseBody: string`。正文均不允许 null，空正文为 `""`。

日志仅在内存保留最近 200 条，离开页面或停止服务不会清除，退出应用后清除。请求正文最多保留前 64 KiB；非 UTF-8 内容用替代字符显示。Authorization、Proxy-Authorization、Cookie、X-API-Key，以及头名称包含 token / secret 的值显示为 `[已隐藏]`。URL、正文和固定响应不脱敏，仅发送虚构测试数据；不会将这些请求写入 Cookie Bridge 文件日志。

配置保存到 Tauri 应用数据目录的 `http-test-server.json`，不读写现有 SQLite 数据。导出文件名称为 `HTTP测试服务-日期时间-随机数.md`，使用独占创建避免覆盖文件。

## 其他入口及本次差异

- 现有 8765 HTTP、Cookie Bridge、域名和任务分支接口没有变更。
- 新增服务由桌面 IPC 管理；没有新的远程管理 HTTP API 或 CLI 子命令，CLI 不会自动启动此服务。
- 新增两个页面 `/http-test-server`、`/http-test-logs`；浏览器预览支持配置与 MD 预览，原生能力按钮明确不可用。
- 原文档没有本模块契约，本次按新增 Rust DTO、HTTP 处理器和 Vue IPC 消费补齐，未修改其他模块字段命名。
