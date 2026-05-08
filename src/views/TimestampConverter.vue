<template>
  <div class="timestamp-converter">
    <div class="tool-header">
      <div class="tool-header-left">
        <h1>时间戳转换工具</h1>
        <p>将服务器时间戳（毫秒/秒）转换为 yyyy-MM-dd HH:mm:ss 格式</p>
      </div>
      <div class="tool-actions">
        <button class="btn btn-ghost" @click="fillCurrent" title="填入当前时间戳">
          <span class="material-icons">schedule</span>
          <span>当前时间</span>
        </button>
      </div>
    </div>

    <div class="content">
      <div class="panel">
        <div class="section-header">
          <h3><span class="material-icons">edit</span> 输入时间戳</h3>
        </div>
        <div class="panel-body">
          <div class="input-row">
            <input
              v-model="timestampInput"
              type="text"
              inputmode="numeric"
              class="ts-input"
              placeholder="请输入时间戳"
              @keyup.enter="convert"
            />
            <select v-model="unit" class="ts-select">
              <option value="ms">毫秒 (ms)</option>
              <option value="s">秒 (s)</option>
            </select>
            <button class="btn btn-primary" @click="convert">转换</button>
            <button class="btn btn-ghost" @click="clearAll">清空</button>
          </div>

          <div class="quick-fill">
            <span class="quick-label">快捷填入：</span>
            <button class="chip" @click="setExample('now-ms')">当前(ms)</button>
            <button class="chip" @click="setExample('now-s')">当前(s)</button>
            <button class="chip" @click="pasteFromClipboard">从剪贴板粘贴</button>
          </div>

          <div v-if="errorMessage" class="error-message">{{ errorMessage }}</div>
        </div>
      </div>

      <div class="panel">
        <div class="section-header">
          <h3><span class="material-icons">schedule</span> 转换结果</h3>
          <div class="actions">
            <button class="btn btn-ghost" :disabled="!formattedDate" @click="copyResult">
              <span class="material-icons">content_copy</span>
              <span>复制</span>
            </button>
          </div>
        </div>
        <div class="panel-body result-body">
          <div v-if="formattedDate" class="result-block">
            <div class="result-label">本地时间 (yyyy-MM-dd HH:mm:ss)</div>
            <div class="result-value primary">
              <span>{{ dateParts.yearPrefix }}</span>
              <span class="year-highlight">{{ dateParts.yearSuffix }}</span>
              <span>{{ dateParts.sep1 }}</span>
              <span class="month-highlight">{{ dateParts.month }}</span>
              <span>{{ dateParts.rest }}</span>
            </div>
          </div>
          <div v-else class="placeholder">
            <span class="material-icons placeholder-icon">access_time</span>
            <p>请输入时间戳并点击转换</p>
          </div>
        </div>
      </div>
    </div>

    <div v-if="toast" class="toast" :class="toast.type">{{ toast.message }}</div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'

type Unit = 'ms' | 's'

const timestampInput = ref('')
const unit = ref<Unit>('ms')
const formattedDate = ref('')
const utcDate = ref('')
const msTimestamp = ref('')
const sTimestamp = ref('')
const weekday = ref('')
const timezone = ref('')
const errorMessage = ref('')
const toast = ref<{ type: 'success' | 'error' | 'info'; message: string } | null>(null)

function pad(n: number): string {
  return n < 10 ? `0${n}` : `${n}`
}

function formatLocal(date: Date): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
}

function formatUtc(date: Date): string {
  return `${date.getUTCFullYear()}-${pad(date.getUTCMonth() + 1)}-${pad(date.getUTCDate())} ${pad(date.getUTCHours())}:${pad(date.getUTCMinutes())}:${pad(date.getUTCSeconds())}`
}

function getWeekday(date: Date): string {
  const days = ['星期日', '星期一', '星期二', '星期三', '星期四', '星期五', '星期六']
  return days[date.getDay()]
}

function getTimezone(): string {
  const offset = -new Date().getTimezoneOffset()
  const sign = offset >= 0 ? '+' : '-'
  const abs = Math.abs(offset)
  return `UTC${sign}${pad(Math.floor(abs / 60))}:${pad(abs % 60)}`
}

function reset() {
  formattedDate.value = ''
  utcDate.value = ''
  msTimestamp.value = ''
  sTimestamp.value = ''
  weekday.value = ''
  timezone.value = ''
}

function convert() {
  errorMessage.value = ''
  const raw = timestampInput.value.trim()
  if (!raw) {
    reset()
    errorMessage.value = '请输入时间戳'
    return
  }

  if (!/^-?\d+(\.\d+)?$/.test(raw)) {
    reset()
    errorMessage.value = '时间戳必须是数字'
    return
  }

  const num = Number(raw)
  if (!Number.isFinite(num)) {
    reset()
    errorMessage.value = '时间戳格式不正确'
    return
  }

  const ms = unit.value === 'ms' ? num : num * 1000
  const date = new Date(ms)
  if (Number.isNaN(date.getTime())) {
    reset()
    errorMessage.value = '无法解析为有效时间'
    return
  }

  formattedDate.value = formatLocal(date)
  utcDate.value = formatUtc(date)
  msTimestamp.value = String(Math.trunc(ms))
  sTimestamp.value = String(Math.trunc(ms / 1000))
  weekday.value = getWeekday(date)
  timezone.value = getTimezone()
}

const canAutoConvert = computed(() => timestampInput.value.trim() !== '')

const dateParts = computed(() => {
  if (!formattedDate.value) return { yearPrefix: '', yearSuffix: '', sep1: '', month: '', rest: '' }
  const s = formattedDate.value
  return {
    yearPrefix: s.slice(0, 2),
    yearSuffix: s.slice(2, 4),
    sep1: s.slice(4, 5),
    month: s.slice(5, 7),
    rest: s.slice(7),
  }
})

watch([timestampInput, unit], () => {
  if (canAutoConvert.value) {
    convert()
  } else {
    reset()
    errorMessage.value = ''
  }
})

function fillCurrent() {
  const now = Date.now()
  unit.value = 'ms'
  timestampInput.value = String(now)
}

function setExample(kind: 'now-ms' | 'now-s') {
  const now = Date.now()
  if (kind === 'now-ms') {
    unit.value = 'ms'
    timestampInput.value = String(now)
  } else {
    unit.value = 's'
    timestampInput.value = String(Math.floor(now / 1000))
  }
}

async function pasteFromClipboard() {
  try {
    const text = await navigator.clipboard.readText()
    if (!text) {
      showToast('error', '剪贴板为空')
      return
    }
    timestampInput.value = text.trim()
    showToast('success', '已从剪贴板粘贴')
  } catch (err) {
    showToast('error', '读取剪贴板失败')
  }
}

async function copyResult() {
  if (!formattedDate.value) return
  try {
    await navigator.clipboard.writeText(formattedDate.value)
    showToast('success', '已复制到剪贴板')
  } catch (err) {
    showToast('error', '复制失败')
  }
}

function clearAll() {
  timestampInput.value = ''
  reset()
  errorMessage.value = ''
}

function showToast(type: 'success' | 'error' | 'info', message: string) {
  toast.value = { type, message }
  setTimeout(() => {
    toast.value = null
  }, 2000)
}
</script>

<style scoped>
.timestamp-converter {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 20px 24px 24px;
  gap: 16px;
  overflow: auto;
}

.tool-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
}

.tool-header-left h1 {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0 0 6px 0;
  letter-spacing: -0.02em;
  line-height: 1.2;
}

.tool-header-left p {
  font-size: 14px;
  color: var(--text-muted);
  margin: 0;
}

.tool-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.content {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.panel {
  display: flex;
  flex-direction: column;
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--border-default);
  border-top-color: var(--border-hover);
  border-radius: 12px;
  box-shadow: inset 0 1px 0 0 rgba(255,255,255,0.05);
  overflow: hidden;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px 18px;
  background: rgba(255,255,255,0.03);
  border-bottom: 1px solid var(--border-default);
  flex-shrink: 0;
}

.section-header h3 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-header h3 .material-icons {
  font-size: 18px;
  color: var(--accent-primary);
}

.actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  align-items: center;
}

.panel-body {
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.input-row {
  display: flex;
  gap: 10px;
  align-items: stretch;
  flex-wrap: wrap;
}

.ts-input {
  flex: 1;
  min-width: 240px;
  padding: 10px 14px;
  font-size: 14px;
  font-family: 'SF Mono', 'Monaco', monospace;
  background: rgba(0,0,0,0.20);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  color: var(--text-primary);
  outline: none;
  transition: border-color 0.2s ease;
}

.ts-input:focus {
  border-color: var(--accent-primary);
}

.ts-select {
  padding: 10px 12px;
  font-size: 13px;
  background: rgba(0,0,0,0.20);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  color: var(--text-primary);
  outline: none;
  cursor: pointer;
  min-width: 130px;
}

.ts-select:focus {
  border-color: var(--accent-primary);
}

.quick-fill {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}

.quick-label {
  font-size: 12px;
  color: var(--text-muted);
}

.chip {
  display: inline-flex;
  align-items: center;
  padding: 5px 12px;
  border-radius: 999px;
  font-size: 12px;
  border: 1px solid var(--border-default);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.chip:hover {
  background: rgba(255,255,255,0.05);
  border-color: var(--border-hover);
  color: var(--text-primary);
}

.error-message {
  padding: 10px 14px;
  border-radius: 8px;
  background: rgba(255, 99, 132, 0.10);
  border: 1px solid rgba(255, 99, 132, 0.30);
  color: #ff8da1;
  font-size: 13px;
}

.result-body {
  gap: 16px;
}

.result-block {
  padding: 18px 20px;
  border-radius: 10px;
  background: rgba(214, 186, 255, 0.08);
  border: 1px solid rgba(214, 186, 255, 0.20);
}

.result-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 12px;
}

.result-item {
  padding: 12px 14px;
  border-radius: 8px;
  background: rgba(255,255,255,0.03);
  border: 1px solid var(--border-default);
}

.result-label {
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  margin-bottom: 6px;
}

.result-value {
  font-family: 'SF Mono', 'Monaco', monospace;
  font-size: 14px;
  color: var(--text-primary);
  word-break: break-all;
}

.result-value.primary {
  font-size: 22px;
  font-weight: 600;
  color: var(--accent-primary);
  letter-spacing: 0.02em;
}

.year-highlight {
  color: #ff4d4f;
  font-weight: 700;
}

.month-highlight {
  color: #fadb14;
  font-weight: 700;
}

.placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 36px 16px;
  color: var(--text-muted);
}

.placeholder-icon {
  font-size: 40px;
  opacity: 0.4;
}

.placeholder p {
  margin: 0;
  font-size: 13px;
}

.btn .material-icons {
  font-size: 16px;
}

.toast {
  position: fixed;
  right: 24px;
  bottom: 24px;
  padding: 10px 16px;
  border-radius: 8px;
  font-size: 13px;
  background: rgba(20, 20, 22, 0.95);
  border: 1px solid var(--border-default);
  color: var(--text-primary);
  box-shadow: 0 4px 14px rgba(0,0,0,0.30);
  z-index: 100;
}

.toast.success {
  border-color: rgba(39, 174, 96, 0.40);
  color: #b5cea8;
}

.toast.error {
  border-color: rgba(255, 99, 132, 0.40);
  color: #ff8da1;
}
</style>
