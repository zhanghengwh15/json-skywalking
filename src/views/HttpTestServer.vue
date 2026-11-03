<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { defaultTestConfig, methods, newEndpoint, testServerMarkdown, validateTestConfig } from '../utils/httpTestServer'
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
    if (initial) config.value = status.config
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
  if (!desktop) return
  await refreshStatus(true)
  if (!disposed) timer = setInterval(() => { void refreshStatus(!ready.value) }, 1500)
})
onUnmounted(() => { disposed = true; if (timer) clearInterval(timer) })
</script>

<template>
  <div class="test-tool">
    <header class="test-header">
      <div><h1>测试服务</h1><p>绑定本机端口，接收任意参数并返回固定 JSON</p></div>
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
      <article v-for="(endpoint, index) in config.endpoints" :key="index" class="test-endpoint">
        <div class="test-endpoint-fields">
          <label>请求方法<select v-model="endpoint.method" :disabled="running || busy || !ready"><option v-for="method in methods" :key="method">{{ method }}</option></select></label>
          <label class="test-grow">URL 路径<input v-model="endpoint.path" placeholder="/test" :disabled="running || busy || !ready" /></label>
          <label>状态码<input v-model.number="endpoint.statusCode" type="number" min="200" max="599" :disabled="running || busy || !ready" /></label>
          <button class="btn btn-ghost" :disabled="running || busy || !ready || config.endpoints.length <= 1" @click="config.endpoints.splice(index, 1)">删除</button>
        </div>
        <code class="test-url">http://127.0.0.1:{{ config.port }}{{ endpoint.path }}</code>
        <label>固定 JSON 响应<textarea v-model="endpoint.responseBody" rows="6" spellcheck="false" :disabled="running || busy || !ready" /></label>
        <p class="test-help">入参无必填项，支持查询参数、JSON、表单、文本和空请求体。ANY 接受任意方法，OPTIONS 自动返回 204。</p>
        <p v-if="endpoint.method === 'HEAD' || [204, 205, 304].includes(endpoint.statusCode)" class="test-help">当前方法或状态码按 HTTP 规则不返回响应体。</p>
      </article>
    </section>
    <details class="test-panel test-markdown"><summary>MD 使用说明预览（随当前配置更新）</summary><pre>{{ markdown }}</pre></details>
    <p class="test-help">请求日志仅保留当前应用会话最近 200 条；正文最多展示 64 KiB，请求体上限 1 MiB。关闭窗口进入托盘后服务继续运行。</p>
  </div>
</template>
