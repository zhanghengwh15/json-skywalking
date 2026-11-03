<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { useRoute } from 'vue-router'
import type { RequestLogSummary, TestRequestLog } from '../utils/httpTestServer'
import '../assets/styles/HttpTestServer.css'

const route = useRoute()
const desktop = isTauri()
const logs = ref<RequestLogSummary[]>([])
const selected = ref<TestRequestLog | null>(null)
const keyword = ref(typeof route.query.path === 'string' ? route.query.path : '')
const method = ref('ALL')
const autoRefresh = ref(true)
const busy = ref(false)
const clearing = ref(false)
const detailBusy = ref(false)
const error = ref('')
let disposed = false
let selectionVersion = 0
let timer: ReturnType<typeof setInterval> | undefined
const filtered = computed(() => logs.value.filter(log =>
  (method.value === 'ALL' || log.method === method.value) &&
  `${log.url} ${log.statusCode} ${log.port}`.toLowerCase().includes(keyword.value.trim().toLowerCase())))

function timestamp(value: string) { return new Date(value).toLocaleString('zh-CN', { hour12: false }) }
function displayBody(value: string) {
  if (!value) return '（空）'
  try { return JSON.stringify(JSON.parse(value), null, 2) } catch { return value }
}

async function refresh() {
  if (!desktop || busy.value || clearing.value || disposed) return
  busy.value = true
  try {
    const result = await invoke<RequestLogSummary[]>('http_test_server_logs')
    if (disposed) return
    logs.value = result
    if (selected.value && !result.some(log => log.id === selected.value?.id)) selected.value = null
    error.value = ''
  } catch (cause) { if (!disposed) error.value = '加载日志失败：' + String(cause) }
  finally { busy.value = false }
}

async function selectLog(id: number) {
  const version = ++selectionVersion
  detailBusy.value = true
  error.value = ''
  try {
    const log = await invoke<TestRequestLog | null>('http_test_server_log', { id })
    if (disposed || version !== selectionVersion) return
    selected.value = log
    if (!log) error.value = '该日志已清空或超出最近 200 条保留范围，请刷新列表。'
  } catch (cause) { if (!disposed && version === selectionVersion) error.value = '加载详情失败：' + String(cause) }
  finally { if (version === selectionVersion) detailBusy.value = false }
}

async function clearLogs() {
  if (clearing.value || busy.value) return
  clearing.value = true
  ++selectionVersion
  detailBusy.value = false
  try {
    await invoke('http_test_server_clear_logs')
    if (!disposed) { logs.value = []; selected.value = null; error.value = '' }
  } catch (cause) { if (!disposed) error.value = '清空失败：' + String(cause) }
  finally { clearing.value = false }
}

onMounted(() => {
  if (!desktop) return
  void refresh()
  timer = setInterval(() => { if (autoRefresh.value) void refresh() }, 1000)
})
onUnmounted(() => { disposed = true; ++selectionVersion; if (timer) clearInterval(timer) })
</script>

<template>
  <div class="test-tool test-logs-page">
    <header class="test-header">
      <div><h1>请求日志</h1><p>查看测试服务接收的请求及实际返回结果</p></div>
      <div class="test-actions">
        <router-link class="btn btn-ghost" to="/http-test-server">测试服务</router-link>
        <button class="btn" :disabled="!desktop || busy || clearing" @click="refresh">{{ busy ? '刷新中…' : '刷新' }}</button>
        <button class="btn btn-ghost" :disabled="!desktop || busy || clearing || !logs.length" @click="clearLogs">{{ clearing ? '清空中…' : '清空日志' }}</button>
      </div>
    </header>
    <p v-if="!desktop" class="test-message">请求日志需在 Dev Tools 桌面应用中查看。</p>
    <p v-if="error" class="test-message test-error" role="alert">{{ error }}</p>
    <div class="test-filters">
      <input v-model="keyword" aria-label="筛选日志" placeholder="筛选 URL、端口或状态码" />
      <select v-model="method" aria-label="筛选请求方法"><option value="ALL">全部方法</option><option v-for="value in ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS']" :key="value">{{ value }}</option></select>
      <label class="test-checkbox"><input v-model="autoRefresh" type="checkbox" />每秒刷新</label>
      <span class="test-help">显示 {{ filtered.length }} / {{ logs.length }} 条</span>
    </div>
    <div class="test-log-layout">
      <section class="test-panel test-log-list">
        <div class="test-table-scroll"><table><thead><tr><th>时间 / 方法</th><th>URL</th><th>状态 / 耗时</th></tr></thead>
          <tbody>
            <tr v-for="log in filtered" :key="log.id" :class="{ selected: selected?.id === log.id }">
              <td><small>{{ timestamp(log.timestamp) }}</small><strong>{{ log.method }}</strong></td>
              <td><button class="test-log-link" :disabled="clearing" @click="selectLog(log.id)">{{ log.url }}</button><small>{{ log.remoteAddress }}</small></td>
              <td><strong :class="log.statusCode >= 400 ? 'test-error' : 'test-success'">{{ log.statusCode }}</strong><small>{{ log.durationMs }} ms</small></td>
            </tr>
            <tr v-if="!filtered.length"><td colspan="3" class="test-empty">{{ logs.length ? '没有符合筛选条件的请求' : '暂无请求，请先启动服务并向配置的 URL 发送数据' }}</td></tr>
          </tbody>
        </table></div>
      </section>
      <section class="test-panel test-log-detail">
        <p v-if="detailBusy" role="status">正在加载详情…</p>
        <template v-else-if="selected">
          <h2>请求 #{{ selected.id }}</h2>
          <dl><dt>时间</dt><dd>{{ timestamp(selected.timestamp) }}</dd><dt>URL</dt><dd>{{ selected.method }} {{ selected.url }}</dd><dt>来源</dt><dd>{{ selected.remoteAddress }}</dd><dt>匹配路径</dt><dd>{{ selected.matchedPath ?? '未匹配（含预检请求）' }}</dd><dt>返回结果</dt><dd>HTTP {{ selected.statusCode }} · {{ selected.durationMs }} ms</dd></dl>
          <h3>请求头</h3><pre>{{ selected.headers.map(([key, value]) => `${key}: ${value}`).join('\n') || '（空）' }}</pre>
          <h3>请求体</h3><p v-if="selected.bodyTruncated" class="test-message">正文已截断，或因超限、读取失败、超时未能完整记录。</p><pre>{{ displayBody(selected.body) }}</pre>
          <h3>响应体</h3><pre>{{ displayBody(selected.responseBody) }}</pre>
        </template>
        <p v-else class="test-empty">点击左侧 URL 查看请求和响应详情</p>
      </section>
    </div>
    <p class="test-help">仅保留当前应用会话最近 200 条，退出应用后清除。请求头中的常见凭据会隐藏，URL 和正文保留测试原文；非 UTF-8 内容以替代字符显示。</p>
  </div>
</template>
