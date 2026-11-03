<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { formatCurl, type CurlBodyOption, type CurlFormatResult } from '../utils/curlFormatter'

const input = ref('')
const bodyOption = ref<CurlBodyOption>('--data-binary')
const multiline = ref(true)
const addJsonContentType = ref(true)
const result = ref<CurlFormatResult | null>(null)
const error = ref('')
const feedback = ref('')
let feedbackTimer: ReturnType<typeof setTimeout> | undefined
const canConvert = computed(() => input.value.trim().length > 0)

function notify(message: string): void {
  feedback.value = message
  clearTimeout(feedbackTimer)
  feedbackTimer = setTimeout(() => { feedback.value = '' }, 3500)
}

function convert(): void {
  error.value = ''
  feedback.value = ''
  result.value = null
  try {
    result.value = formatCurl(input.value, {
      bodyOption: bodyOption.value, multiline: multiline.value,
      addJsonContentType: addJsonContentType.value,
    })
  } catch (cause: unknown) {
    error.value = cause instanceof Error ? cause.message : '转换失败，请检查命令格式。'
  }
}

watch(input, () => { result.value = null; error.value = ''; feedback.value = '' }, { flush: 'sync' })
watch([bodyOption, multiline, addJsonContentType], () => { if (canConvert.value) convert() })
onUnmounted(() => clearTimeout(feedbackTimer))

function clear(): void { input.value = ''; result.value = null; error.value = ''; feedback.value = '' }

function fillExample(): void {
  input.value = [
    "curl 'https://example.com/api/query'",
    "  -H 'Content-Type: application/json'",
    `  --json '{"sql":"SELECT * FROM orders WHERE status = '\\''ready'\\'' AND id = 9007199254740993","limit":10}'`,
  ].join(' \\\n')
  convert()
}

async function paste(): Promise<void> {
  try { input.value = await navigator.clipboard.readText() }
  catch { notify('无法读取剪贴板，请在输入框中按 Ctrl / ⌘ + V 粘贴。') }
}

async function copy(): Promise<void> {
  if (!result.value) return
  try {
    await navigator.clipboard.writeText(result.value.command)
    notify('已复制 Linux 命令。')
  } catch { notify('复制失败，请在结果框中全选并手动复制。') }
}
</script>

<template>
  <section class="curl-formatter">
    <div class="formatter-header">
      <div>
        <h1>cURL 格式化</h1>
        <p>将浏览器导出的请求转为旧版 Linux 可用的命令，保留 JSON、SQL 和原始请求内容。</p>
      </div>
      <span class="compatibility-badge"><span class="material-icons">terminal</span>cURL 7.29 兼容目标</span>
    </div>

    <div class="formatter-options">
      <label class="body-option">请求体参数
        <select v-model="bodyOption" aria-label="请求体参数">
          <option value="--data-binary">--data-binary（保留原始正文）</option>
          <option value="-d">-d（常用写法）</option>
        </select>
      </label>
      <label class="check-option"><input v-model="multiline" type="checkbox" />多行排版</label>
      <label class="check-option"><input v-model="addJsonContentType" type="checkbox" />补齐 JSON Content-Type</label>
      <span class="local-note">本地转换 · 不发送请求 · 不保存凭据</span>
    </div>

    <div class="formatter-panels">
      <div class="formatter-panel input-panel">
        <div class="panel-toolbar">
          <h2><span class="material-icons">edit_note</span>原始 cURL</h2>
          <div class="toolbar-actions">
            <button class="btn btn-ghost" @click="fillExample">示例</button>
            <button class="btn btn-ghost" @click="paste">粘贴</button>
            <button class="btn btn-ghost" :disabled="!input" @click="clear">清空</button>
          </div>
        </div>
        <label class="sr-only" for="curl-source">原始 cURL 命令</label>
        <textarea id="curl-source" v-model="input" spellcheck="false" autocomplete="off" autocapitalize="off"
          placeholder="粘贴 curl 命令，支持 --data-raw、--json、多行续行和 Markdown URL。&#10;&#10;curl 'https://example.com/api' --json '{&quot;name&quot;:&quot;示例&quot;}'"
          @keydown.ctrl.enter.prevent="convert" @keydown.meta.enter.prevent="convert" />
        <div class="panel-footer">
          <span>Ctrl / ⌘ + Enter 转换</span>
          <button class="btn btn-primary" :disabled="!canConvert" @click="convert">
            <span class="material-icons">sync_alt</span>转换为 Linux 命令
          </button>
        </div>
      </div>

      <div class="formatter-panel output-panel">
        <div class="panel-toolbar">
          <h2><span class="material-icons">terminal</span>Linux 命令</h2>
          <button class="btn btn-ghost" :disabled="!result" @click="copy">
            <span class="material-icons">content_copy</span>复制命令
          </button>
        </div>
        <div v-if="error" class="format-error" role="alert"><strong>暂时无法转换</strong><p>{{ error }}</p></div>
        <template v-else-if="result">
          <label class="sr-only" for="curl-result">Linux cURL 命令</label>
          <textarea id="curl-result" :value="result.command" readonly spellcheck="false" aria-label="Linux cURL 命令" />
          <div class="result-stats"><span class="method-tag">{{ result.method }}</span>
            <span>{{ result.headerCount }} 个请求头</span><span>内联正文 {{ result.bodyBytes.toLocaleString() }} 字节</span></div>
        </template>
        <div v-else class="output-placeholder"><span class="material-icons">terminal</span>
          <p>Linux 命令将在这里显示</p><small>保留请求内容，转换新版参数和 Shell 引号。</small></div>
      </div>
    </div>

    <div v-if="feedback" class="clipboard-feedback" role="status">{{ feedback }}</div>
    <div v-if="result && (result.changes.length || result.warnings.length)" class="conversion-notes" aria-live="polite">
      <div v-if="result.changes.length"><h3>本次转换</h3><ul><li v-for="change in result.changes" :key="change">{{ change }}</li></ul></div>
      <div v-if="result.warnings.length" class="conversion-warnings"><h3>需要留意</h3><ul><li v-for="warning in result.warnings" :key="warning">{{ warning }}</li></ul></div>
    </div>
    <p class="compatibility-note">目标为 Linux sh / bash 与 cURL 7.29 常用参数。未知参数会提示处理；服务器的网络、证书及 TLS 能力仍以实际环境为准。</p>
  </section>
</template>

<style scoped>
.curl-formatter { height: 100%; overflow: auto; padding: 28px; display: flex; flex-direction: column; gap: 18px; }
.formatter-header { display: flex; align-items: center; justify-content: space-between; gap: 20px; }
h1 { font-size: 24px; margin: 0 0 8px; letter-spacing: -0.02em; }
.formatter-header p { margin: 0; color: var(--text-muted); font-size: 13px; line-height: 1.6; }
.compatibility-badge { display: inline-flex; align-items: center; gap: 7px; white-space: nowrap; padding: 8px 12px; background: rgba(214,186,255,0.08); border: 1px solid rgba(214,186,255,0.2); border-radius: 20px; color: var(--accent-primary); font-size: 12px; }
.compatibility-badge .material-icons { font-size: 16px; }
.formatter-options { display: flex; flex-wrap: wrap; align-items: center; gap: 20px; padding: 14px 16px; background: var(--bg-secondary); border: 1px solid var(--border-default); border-radius: 12px; font-size: 12px; color: var(--text-secondary); }
.body-option, .check-option { display: flex; align-items: center; gap: 8px; }
select { background: var(--bg-elevated); color: var(--text-primary); border: 1px solid var(--border-default); padding: 8px; border-radius: 6px; font: inherit; }
input[type=checkbox] { accent-color: var(--accent-primary); }
.local-note { color: var(--text-muted); margin-left: auto; font-size: 11px; }
.formatter-panels { display: grid; grid-template-columns: 1fr 1fr; gap: 18px; flex: 1; min-height: 380px; }
.formatter-panel { display: flex; flex-direction: column; min-width: 0; overflow: hidden; background: var(--bg-secondary); border: 1px solid var(--border-default); border-radius: 12px; }
.panel-toolbar { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 14px 16px; border-bottom: 1px solid var(--border-subtle); min-height: 65px; }
h2 { display: flex; gap: 8px; align-items: center; margin: 0; font-size: 13px; font-weight: 600; white-space: nowrap; }
h2 .material-icons { color: var(--accent-primary); font-size: 20px; }
.toolbar-actions { display: flex; gap: 6px; }
.toolbar-actions .btn { padding: 6px 10px; }
textarea { display: block; flex: 1; width: 100%; min-height: 280px; resize: none; border: none; outline: none; padding: 18px; background: transparent; color: var(--text-primary); font: 12px/1.8 'SFMono-Regular', Consolas, 'Liberation Mono', monospace; tab-size: 2; white-space: pre; }
textarea::placeholder { color: var(--text-disabled); white-space: pre-wrap; }
textarea:focus-visible { box-shadow: inset 0 0 0 1px var(--accent-primary); }
.output-panel textarea { color: #c9deb4; }
.panel-footer, .result-stats { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 12px 16px; border-top: 1px solid var(--border-subtle); font-size: 11px; color: var(--text-muted); }
.result-stats { justify-content: flex-start; min-height: 60px; }
.method-tag { color: #c9deb4; background: rgba(201,222,180,.08); padding: 4px 7px; border-radius: 4px; font-family: monospace; }
.output-placeholder { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; color: var(--text-muted); padding: 24px; text-align: center; }
.output-placeholder > .material-icons { font-size: 42px; opacity: .4; }
.output-placeholder p { margin: 15px 0 8px; font-size: 13px; }
.output-placeholder small { color: var(--text-disabled); line-height: 1.8; }
.format-error { margin: 18px; padding: 16px; color: #ffb4ab; border: 1px solid rgba(255,180,171,.2); background: rgba(255,180,171,.05); border-radius: 8px; font-size: 13px; line-height: 1.7; overflow-wrap: anywhere; }
.format-error p { margin-bottom: 0; }
.conversion-notes { display: flex; flex-wrap: wrap; gap: 20px 36px; padding: 16px 20px; background: var(--bg-secondary); border: 1px solid var(--border-default); border-radius: 12px; font-size: 12px; color: var(--text-secondary); }
.conversion-notes h3 { margin: 0 0 6px; font-size: 12px; color: var(--accent-primary); }
.conversion-notes ul { margin: 0; padding-left: 18px; line-height: 1.9; }
.conversion-warnings h3 { color: #e8c57c; }
.clipboard-feedback { padding: 10px 16px; border: 1px solid var(--border-default); border-radius: 8px; color: var(--accent-primary); font-size: 12px; }
.compatibility-note { margin: 0; font-size: 11px; color: var(--text-muted); line-height: 1.8; }
.sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0,0,0,0); white-space: nowrap; border: 0; }
@media (max-width: 1100px) { .formatter-header { align-items: flex-start; flex-direction: column; gap: 12px; } .formatter-panels { grid-template-columns: 1fr; } .formatter-panel { min-height: 400px; } .local-note { margin-left: 0; } }
</style>
