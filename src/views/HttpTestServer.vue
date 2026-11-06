<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { defaultTestConfig, methods, newEndpoint, responseContentType, testEndpointCurl, testServerMarkdown, validateTestConfig } from '../utils/httpTestServer'
import type { TestServerStatus } from '../utils/httpTestServer'
import '../assets/styles/HttpTestServer.css'

const desktop = isTauri()
const config = ref(defaultTestConfig())
const running = ref(false)
const ready = ref(!desktop)
const busy = ref(false)
const error = ref('')
const notice = ref('')
const statusError = ref('')
const activeEndpoint = ref(0)
const curlCommand = ref('')
const curlEndpoint = ref(0)
const curlPreview = ref<HTMLElement | null>(null)
let timer: ReturnType<typeof setInterval> | undefined
let disposed = false
let polling = false
const markdown = computed(() => {
  try { return testServerMarkdown(config.value) } catch (cause) { return '请先完善配置：' + String(cause) }
})

async function refreshStatus(initial = false) {
  if (polling || busy.value || disposed) return
  polling = true
  try {
    const status = await invoke<TestServerStatus>('http_test_server_status')
    if (disposed || busy.value) return
    running.value = status.running
    if (initial) config.value = {
      ...status.config,
      endpoints: status.config.endpoints.map(endpoint => ({ ...endpoint, contentType: responseContentType(endpoint) })),
    }
    ready.value = true
    statusError.value = status.lastError ?? ''
  } catch (cause) {
    if (!disposed) statusError.value = '读取服务状态失败：' + String(cause)
  } finally { polling = false }
}

async function toggleServer() {
  if (busy.value || !ready.value) return
  error.value = ''
  notice.value = ''
  busy.value = true
  try {
    if (!running.value) validateTestConfig(config.value)
    const status = await invoke<TestServerStatus>(running.value ? 'http_test_server_stop' : 'http_test_server_start',
      running.value ? {} : { config: config.value })
    if (disposed) return
    running.value = status.running
    statusError.value = status.lastError ?? ''
    notice.value = status.running ? '服务已启动，可以发送测试请求。' : '服务已停止，端口已释放。'
  } catch (cause) {
    if (!disposed) error.value = String(cause)
  } finally { busy.value = false }
}

function addEndpoint() {
  const endpoint = newEndpoint()
  let suffix = config.value.endpoints.length + 1
  while (config.value.endpoints.some(item => item.path === `/test-${suffix}`)) suffix++
  endpoint.path = `/test-${suffix}`
  config.value.endpoints.push(endpoint)
}

function generateCurl(index: number) {
  error.value = ''
  notice.value = ''
  try {
    curlCommand.value = testEndpointCurl(config.value, index)
    curlEndpoint.value = index + 1
    void nextTick(() => curlPreview.value?.scrollIntoView({ block: 'nearest', behavior: 'smooth' }))
  } catch (cause) {
    curlCommand.value = ''
    error.value = '生成 cURL 失败：' + String(cause)
  }
}

async function copyCurl() {
  error.value = ''
  notice.value = ''
  try {
    await navigator.clipboard.writeText(curlCommand.value)
    if (!disposed) notice.value = 'cURL 命令已复制。'
  } catch (cause) {
    if (!disposed) error.value = '复制失败，请从下方命令框手动复制：' + String(cause)
  }
}

function handleShortcut(event: KeyboardEvent) {
  if (!(event.metaKey || event.ctrlKey) || !event.altKey || event.shiftKey || event.code !== 'KeyC' || event.repeat || event.isComposing) return
  event.preventDefault()
  if (!ready.value || busy.value) return
  const index = Math.min(activeEndpoint.value, config.value.endpoints.length - 1)
  generateCurl(index)
}

async function exportMarkdown() {
  if (busy.value) return
  error.value = ''
  notice.value = ''
  busy.value = true
  try {
    const content = testServerMarkdown(config.value)
    const path = await invoke<string>('http_test_server_export_markdown', { content })
    if (!disposed) notice.value = 'MD 文档已保存到：' + path
  } catch (cause) {
    if (!disposed) error.value = '导出失败：' + String(cause)
  } finally { busy.value = false }
}

onMounted(async () => {
  window.addEventListener('keydown', handleShortcut)
  if (!desktop) return
  await refreshStatus(true)
  if (!disposed) timer = setInterval(() => { void refreshStatus(!ready.value) }, 1500)
})
onUnmounted(() => { disposed = true; window.removeEventListener('keydown', handleShortcut); if (timer) clearInterval(timer) })
</script>

<template>
  <div class="test-tool">
    <header class="test-header">
      <div><h1>测试服务</h1><p>绑定本机端口，接收任意参数并返回自定义内容</p></div>
      <div class="test-actions">
        <router-link class="btn btn-ghost" to="/http-test-logs">查看请求日志</router-link>
        <button class="btn" :disabled="!desktop || !ready || busy" @click="exportMarkdown">导出 MD</button>
        <button class="btn btn-primary" :disabled="!desktop || !ready || busy" @click="toggleServer">
          {{ busy ? '处理中…' : running ? '停止服务' : '启动服务' }}
        </button>
      </div>
    </header>
    <p v-if="!desktop" class="test-message">浏览器仅可预览配置和文档。启动监听、查看日志和导出文件请使用 Dev Tools 桌面应用。</p>
    <p v-if="error || statusError" class="test-message test-error" role="alert">{{ error || statusError }}</p>
    <p v-if="notice" class="test-message test-success" role="status">{{ notice }}</p>

    <section class="test-panel test-port">
      <span class="test-status" :class="{ running }">{{ !ready ? '连接中' : running ? '运行中' : '已停止' }}</span>
      <label>监听地址 <input value="127.0.0.1" disabled /></label>
      <label>端口 <input v-model.number="config.port" type="number" min="1024" max="65535" :disabled="running || busy || !ready" /></label>
      <p>仅本机访问 · 启动成功后保存配置 · 修改配置请先停止服务</p>
    </section>

    <section class="test-panel">
      <div class="test-section-title"><h2>接口配置 <span>{{ config.endpoints.length }} / 50</span></h2>
        <button class="btn" :disabled="running || busy || !ready || config.endpoints.length >= 50" @click="addEndpoint">添加 URL</button>
      </div>
      <p class="test-help">点击接口内「生成 cURL」，或聚焦接口后按 ⌘ / Ctrl + Alt + C；未选择时使用第一个接口。</p>
      <article v-for="(endpoint, index) in config.endpoints" :key="index" class="test-endpoint" @focusin="activeEndpoint = index" @click="activeEndpoint = index">
        <div class="test-endpoint-fields">
          <label>请求方法<select v-model="endpoint.method" :disabled="running || busy || !ready"><option v-for="method in methods" :key="method">{{ method }}</option></select></label>
          <label class="test-grow">URL 路径<input v-model="endpoint.path" placeholder="/test" :disabled="running || busy || !ready" /></label>
          <label>状态码<input v-model.number="endpoint.statusCode" type="number" min="200" max="599" :disabled="running || busy || !ready" /></label>
          <button class="btn btn-ghost" :disabled="running || busy || !ready || config.endpoints.length <= 1" @click="config.endpoints.splice(index, 1)">删除</button>
        </div>
        <div class="test-url-row"><code class="test-url">http://127.0.0.1:{{ config.port }}{{ endpoint.path }}</code>
          <button class="btn test-curl-button" :disabled="busy || !ready" @click="generateCurl(index)"><span aria-hidden="true">⌘</span> 生成 cURL</button>
        </div>
        <label>响应类型（Content-Type）<input v-model="endpoint.contentType" placeholder="application/json; charset=utf-8" :disabled="running || busy || !ready" /></label>
        <label class="test-response-body">自定义响应体<textarea v-model="endpoint.responseBody" rows="6" spellcheck="false" :disabled="running || busy || !ready" /></label>
        <p class="test-help">默认提供 code、message、data 示例，可全部替换为任意 JSON、文本、HTML、XML 或空内容；按原文返回，最多 64 KiB。</p>
        <p v-if="running" class="test-help">服务运行中，响应配置已锁定；请先点击「停止服务」，修改后重新启动生效。</p>
        <p class="test-help">入参无必填项，支持查询参数、JSON、表单、文本和空请求体。ANY 接受任意方法，OPTIONS 自动返回 204。</p>
        <p v-if="endpoint.method === 'HEAD' || [204, 205, 304].includes(endpoint.statusCode)" class="test-help">当前方法或状态码按 HTTP 规则不返回响应体。</p>
      </article>
    </section>
    <section v-if="curlCommand" ref="curlPreview" class="test-panel test-curl-preview" aria-label="cURL 命令预览">
      <div class="test-section-title"><div><h2>接口 {{ curlEndpoint }} · cURL</h2><p class="test-help">根据生成时的配置创建，可直接在终端执行；修改配置后请重新生成。</p></div>
        <button class="btn test-curl-button" @click="copyCurl">复制命令</button>
      </div>
      <textarea :value="curlCommand" readonly rows="3" aria-label="生成的 cURL 命令" spellcheck="false" />
    </section>
    <details class="test-panel test-markdown"><summary><span class="test-disclosure-icon" aria-hidden="true">›</span><span class="test-doc-icon" aria-hidden="true">MD</span><span>使用说明预览<small>随当前配置更新</small></span><span class="test-disclosure-label">展开</span></summary><pre>{{ markdown }}</pre></details>
    <p class="test-help">请求日志仅保留当前应用会话最近 200 条；正文最多展示 64 KiB，请求体上限 1 MiB。关闭窗口进入托盘后服务继续运行。</p>
  </div>
</template>
