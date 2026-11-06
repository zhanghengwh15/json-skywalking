export const methods = ['ANY', 'GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD'] as const

export interface TestEndpoint {
  method: string
  path: string
  statusCode: number
  responseBody: string
  contentType?: string
}

export interface TestServerConfig {
  port: number
  endpoints: TestEndpoint[]
}

export interface TestServerStatus {
  running: boolean
  config: TestServerConfig
  lastError: string | null
}

export interface RequestLogSummary {
  id: number
  timestamp: string
  port: number
  method: string
  url: string
  remoteAddress: string
  statusCode: number
  durationMs: number
  matchedPath: string | null
}

export interface TestRequestLog extends RequestLogSummary {
  headers: [string, string][]
  body: string
  bodyTruncated: boolean
  responseBody: string
}

export function newEndpoint(): TestEndpoint {
  return {
    method: 'ANY',
    path: '/test',
    statusCode: 200,
    responseBody: JSON.stringify({ code: 0, message: 'ok', data: null }, null, 2),
    contentType: 'application/json; charset=utf-8',
  }
}

export function defaultTestConfig(): TestServerConfig {
  return { port: 8766, endpoints: [newEndpoint()] }
}

export function validateTestConfig(config: TestServerConfig): void {
  if (!Number.isInteger(config.port) || config.port < 1024 || config.port > 65535 || config.port === 8765) {
    throw new Error('端口需为 1024–65535，且不能使用 Cookie Bridge 的 8765')
  }
  if (!config.endpoints.length || config.endpoints.length > 50) throw new Error('请配置 1–50 个接口')
  config.endpoints.forEach((endpoint, index) => {
    const prefix = `接口 ${index + 1}：`
    if (!methods.some(method => method === endpoint.method)) throw new Error(prefix + '请求方法不支持')
    if (!endpoint.path.startsWith('/') || endpoint.path.startsWith('//') || endpoint.path.length > 2048 || /[^\x21-\x7e]|[?#]/.test(endpoint.path)) {
      throw new Error(prefix + '填写以 / 开头的 URL 路径，不含查询参数、片段或空格；中文请 URL 编码')
    }
    if (!Number.isInteger(endpoint.statusCode) || endpoint.statusCode < 200 || endpoint.statusCode > 599) {
      throw new Error(prefix + '状态码需为 200–599')
    }
    if (new TextEncoder().encode(endpoint.responseBody).length > 64 * 1024) throw new Error(prefix + '固定响应不能超过 64 KiB')
    const contentType = responseContentType(endpoint)
    if (!contentType.trim() || contentType.length > 256 || /[^\x20-\x7e]/.test(contentType)) {
      throw new Error(prefix + '响应类型需为 1–256 个可打印 ASCII 字符，不能全为空格或包含换行')
    }
    if (config.endpoints.slice(0, index).some(previous => previous.path === endpoint.path &&
      (previous.method === endpoint.method || previous.method === 'ANY' || endpoint.method === 'ANY'))) {
      throw new Error(prefix + '路径与方法重复或与 ANY 冲突')
    }
  })
}

export function responseContentType(endpoint: TestEndpoint): string {
  return endpoint.contentType ?? 'application/json; charset=utf-8'
}

function fenced(text: string, language = ''): string {
  const ticks = Math.max(3, ...(text.match(/`+/g) ?? []).map(value => value.length + 1))
  const fence = '`'.repeat(ticks)
  return `${fence}${language}\n${text}\n${fence}`
}

function shellQuote(value: string): string {
  return "'" + value.replace(/'/g, "'\"'\"'") + "'"
}

export function testEndpointCurl(config: TestServerConfig, index: number): string {
  const endpoint = config.endpoints[index]
  if (!endpoint) throw new Error('请选择要生成 cURL 的接口')
  validateTestConfig({ port: config.port, endpoints: [endpoint] })
  const url = `http://127.0.0.1:${config.port}${endpoint.path}`
  const method = endpoint.method === 'ANY' ? 'POST' : endpoint.method
  if (method === 'HEAD') return `curl --head --url ${shellQuote(url)}`
  return `curl -X ${method} --url ${shellQuote(url)} -H 'Content-Type: application/json' --data-binary '{"example":"任意值","count":1}'`
}

export function testServerMarkdown(config: TestServerConfig): string {
  validateTestConfig(config)
  const base = `http://127.0.0.1:${config.port}`
  const sections = config.endpoints.map((endpoint, index) => {
    const url = base + endpoint.path
    const method = endpoint.method === 'ANY' ? 'POST' : endpoint.method
    const curl = testEndpointCurl(config, index)
    const noBody = method === 'HEAD' || [204, 205, 304].includes(endpoint.statusCode)
    return [
      `## 接口 ${index + 1}`,
      fenced(`${endpoint.method} ${url}`, 'text'),
      endpoint.method === 'ANY' ? '方法：接受任意 HTTP 方法；OPTIONS 用于自动预检，返回 204。' : `方法：${endpoint.method}。`,
      '入参：无必填字段，查询参数、请求头及 JSON、表单、文本或空请求体均可任意填写，不做业务校验。URL 路径和方法精确匹配，查询参数不影响匹配。',
      `自定义响应：HTTP ${endpoint.statusCode}，Content-Type 为 ${responseContentType(endpoint)}。`,
      noBody ? '此状态码或 HEAD 方法按 HTTP 规则不返回响应体。' : fenced(endpoint.responseBody, 'text'),
      '调用示例（macOS / Linux shell；Windows 可使用 Git Bash 或 WSL）：',
      fenced(curl, 'bash'),
    ].join('\n\n')
  })
  return [
    '# HTTP 测试服务使用说明',
    '## 启动',
    `在 Dev Tools 桌面应用的「测试服务」页面配置端口 ${config.port} 和下列接口，然后点击「启动服务」。仅本机访问，基础地址为 ${base}。`,
    '此文档根据当前页面配置生成；编辑后需停止并重新启动服务才能生效。应用重启后不会自动监听。关闭窗口会进入托盘，需停止服务或退出应用才会结束监听。',
    '每个接口的响应体独立配置，默认是 code、message、data 的 JSON 示例，可完整替换为任意 JSON、文本、HTML、XML 或空内容，最多 64 KiB。响应按填写原文返回，不自动包装、格式化或校验 JSON；响应类型可自行修改。',
    ...sections,
    '## 请求日志与边界',
    '打开「请求日志」查看时间、来源、方法、URL、请求头、请求体、状态码、耗时和响应。最近 200 条仅保存在当前应用会话中，退出后清除；停止服务不会清除日志。请求头中的常见凭据会隐藏，正文及 URL 不脱敏，请仅发送测试数据。',
    '请求体上限为 1 MiB，读取超时 15 秒；日志正文只保留前 64 KiB，非 UTF-8 数据以替代字符显示。未匹配的路径或方法返回 404，超限返回 413，读取失败返回 400，超时返回 408。OPTIONS 返回 204。HEAD、204、205 和 304 均不返回响应体。',
    '允许浏览器跨域调用（Access-Control-Allow-Origin: *），不支持携带跨域凭据。此测试服务不要求认证。',
    '## 停止',
    '在「测试服务」点击「停止服务」。若端口占用，请修改端口或先停止占用该端口的程序。',
  ].join('\n\n') + '\n'
}
