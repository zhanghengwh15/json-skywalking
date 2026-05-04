<template>
  <div class="cookie-bridge">
    <div class="tool-header">
      <div class="tool-header-left">
        <h1>Cookie Bridge</h1>
        <p>查看 Chrome 扩展推送的 Cookie 和 LocalStorage 数据</p>
      </div>
      <button
        class="debug-toggle"
        :class="{ active: debugMode }"
        @click="toggleDebug"
        title="切换 Debug 日志模式"
      >
        <span class="material-icons">bug_report</span>
        {{ debugMode ? 'Debug ON' : 'Debug' }}
      </button>
    </div>
    <div class="content">
      <div class="domain-sidebar">
        <div class="panel-header">
          <h3><span class="material-icons">language</span> 域名列表</h3>
        </div>
        <div class="domain-list">
          <ul v-if="domains.length > 0">
            <li
              v-for="d in domains"
              :key="d"
              :class="{ active: d === selectedDomain }"
              @click="selectDomain(d)"
            >
              <span class="domain-dot"></span>
              {{ d }}
            </li>
          </ul>
          <div v-else class="empty">
            等待 Chrome 扩展推送数据
          </div>
        </div>
      </div>
      <div class="detail-panel">
        <div v-if="selectedDomain" class="detail-content">
          <div class="panel-header">
            <h3><span class="material-icons">cookie</span> Cookies</h3>
          </div>
          <div class="table-wrapper">
            <table v-if="currentDomain.cookies.length > 0">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Value</th>
                  <th>Path</th>
                  <th>Expires</th>
                  <th>Secure</th>
                  <th>HttpOnly</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="c in currentDomain.cookies" :key="c.name + c.path">
                  <td class="mono">{{ c.name }}</td>
                  <td class="mono">{{ c.value }}</td>
                  <td>{{ c.path }}</td>
                  <td>{{ c.expires }}</td>
                  <td>{{ c.secure }}</td>
                  <td>{{ c.httpOnly }}</td>
                </tr>
              </tbody>
            </table>
            <p v-else class="empty-table">无 cookies</p>
          </div>

          <div class="panel-header mt-4">
            <h3><span class="material-icons">storage</span> Local Storage</h3>
          </div>
          <div class="table-wrapper">
            <table v-if="currentDomain.localStorage.length > 0">
              <thead>
                <tr>
                  <th>Key</th>
                  <th>Value</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="ls in currentDomain.localStorage" :key="ls.key">
                  <td class="mono">{{ ls.key }}</td>
                  <td class="mono value-cell">
                    <div class="value-wrap">
                      <span class="value-text">{{ formatValue(ls.value, ls.key) }}</span>
                      <span v-if="ls.value.length > MAX_LENGTH" class="show-more" @click.stop="toggleExpand(ls.key)">
                        {{ isExpanded(ls.key) ? 'show less' : 'show more' }}
                      </span>
                    </div>
                    <button
                      v-if="isJson(ls.value)"
                      class="copy-btn"
                      title="复制"
                      @click.stop="copyText(ls.value)"
                    >
                      <span class="material-icons">content_copy</span>
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
            <p v-else class="empty-table">无 localStorage</p>
          </div>
        </div>
        <div v-else class="empty">
          请从左侧选择一个域名
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const debugMode = ref(false)

async function toggleDebug() {
  debugMode.value = !debugMode.value
  await invoke('cookie_bridge_set_debug_mode', { enabled: debugMode.value })
}

interface CookieItem {
  domain: string
  name: string
  path: string
  value: string
  expires: number
  secure: number
  httpOnly: number
  updatedAt: number
}

interface LocalStorageItem {
  domain: string
  key: string
  value: string
  updatedAt: number
}

interface DomainSnapshot {
  cookies: CookieItem[]
  localStorage: LocalStorageItem[]
}

const domains = ref<string[]>([])
const selectedDomain = ref<string>('')
const currentDomain = ref<DomainSnapshot>({ cookies: [], localStorage: [] })
let unlisten: UnlistenFn | null = null

const MAX_LENGTH = 200
const expandedKeys = ref<Set<string>>(new Set())

function isJson(str: string): boolean {
  try {
    JSON.parse(str)
    return true
  } catch {
    return false
  }
}

function formatValue(value: string, key: string): string {
  if (isExpanded(key)) return value
  return value.length > MAX_LENGTH ? value.slice(0, MAX_LENGTH) + '...' : value
}

function isExpanded(key: string): boolean {
  return expandedKeys.value.has(key)
}

function toggleExpand(key: string) {
  const set = new Set(expandedKeys.value)
  if (set.has(key)) {
    set.delete(key)
  } else {
    set.add(key)
  }
  expandedKeys.value = set
}

async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text)
  } catch (e) {
    console.error('复制失败', e)
  }
}

async function loadDomains() {
  try {
    domains.value = await invoke<string[]>('cookie_bridge_list_domains')
    if (domains.value.length > 0 && !selectedDomain.value) {
      selectedDomain.value = domains.value[0]
      await loadDetail(selectedDomain.value)
    }
  } catch (e) {
    console.error('加载域名列表失败', e)
  }
}

async function loadDetail(domain: string) {
  try {
    currentDomain.value = await invoke<DomainSnapshot>('cookie_bridge_get_domain', { domain })
  } catch (e) {
    console.error('加载域名详情失败', e)
  }
}

async function selectDomain(domain: string) {
  selectedDomain.value = domain
  await loadDetail(domain)
}

onMounted(async () => {
  await loadDomains()
  unlisten = await listen<string>('cookie-bridge:updated', async (event) => {
    await loadDomains()
    if (event.payload === selectedDomain.value) {
      await loadDetail(selectedDomain.value)
    }
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})
</script>

<style scoped>
.cookie-bridge {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 20px 24px 24px;
  gap: 16px;
  overflow: hidden;
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

.debug-toggle {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border-radius: 8px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid var(--border-default);
  background: transparent;
  color: var(--text-muted);
  transition: all 0.2s ease;
}

.debug-toggle:hover {
  background: rgba(255,255,255,0.05);
  border-color: var(--border-hover);
}

.debug-toggle.active {
  background: rgba(214, 186, 255, 0.15);
  border-color: rgba(214, 186, 255, 0.30);
  color: var(--accent-primary);
}

.debug-toggle .material-icons {
  font-size: 16px;
}

.content {
  display: grid;
  grid-template-columns: 260px 1fr;
  gap: 16px;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.domain-sidebar {
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

.panel-header {
  padding: 14px 18px;
  background: rgba(255,255,255,0.03);
  border-bottom: 1px solid var(--border-default);
  flex-shrink: 0;
}

.panel-header h3 {
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

.panel-header h3 .material-icons {
  font-size: 18px;
  color: var(--accent-primary);
}

.mt-4 {
  margin-top: 16px;
}

.domain-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.domain-list ul {
  list-style: none;
  padding: 0;
  margin: 0;
}

.domain-list li {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  cursor: pointer;
  border-radius: 8px;
  font-size: 13px;
  color: var(--text-secondary);
  transition: all 0.15s ease;
}

.domain-list li:hover {
  background: rgba(255,255,255,0.04);
  color: var(--text-primary);
}

.domain-list li.active {
  background: rgba(214, 186, 255, 0.10);
  color: var(--accent-primary);
  font-weight: 600;
}

.domain-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-muted);
  flex-shrink: 0;
}

.domain-list li.active .domain-dot {
  background: var(--accent-primary);
}

.empty {
  color: var(--text-disabled);
  padding: 24px 16px;
  text-align: center;
  font-size: 13px;
}

.detail-panel {
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

.detail-content {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 16px;
}

.table-wrapper {
  padding: 0 18px;
  overflow-x: auto;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
  margin-top: 8px;
}

th, td {
  border: 1px solid var(--border-subtle);
  padding: 8px 10px;
  text-align: left;
  color: var(--text-secondary);
}

th {
  background: rgba(255,255,255,0.03);
  color: var(--text-primary);
  font-weight: 600;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.03em;
}

td {
  background: rgba(0,0,0,0.05);
}

tbody tr:hover td {
  background: rgba(255,255,255,0.02);
}

.mono {
  font-family: 'SF Mono', 'Monaco', monospace;
}

.empty-table {
  color: var(--text-disabled);
  padding: 16px;
  text-align: center;
  font-style: italic;
  font-size: 13px;
}

.value-cell {
  position: relative;
  padding-right: 32px;
  max-width: 600px;
}

.value-wrap {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.value-text {
  word-break: break-all;
  line-height: 1.5;
}

.show-more {
  display: inline-flex;
  align-self: flex-start;
  font-size: 11px;
  color: var(--accent-primary);
  cursor: pointer;
  user-select: none;
  padding: 2px 6px;
  border-radius: 4px;
  transition: background 0.15s;
}

.show-more:hover {
  background: rgba(214, 186, 255, 0.10);
}

.copy-btn {
  position: absolute;
  top: 6px;
  right: 6px;
  display: none;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 6px;
  border: none;
  background: rgba(255,255,255,0.08);
  color: var(--text-muted);
  cursor: pointer;
  padding: 0;
  transition: all 0.15s ease;
}

.copy-btn .material-icons {
  font-size: 14px;
}

.copy-btn:hover {
  background: rgba(214, 186, 255, 0.20);
  color: var(--accent-primary);
}

.value-cell:hover .copy-btn {
  display: inline-flex;
}

/* 响应式 */
@media (max-width: 768px) {
  .content {
    grid-template-columns: 1fr;
    grid-template-rows: auto 1fr;
  }

  .cookie-bridge {
    padding: 16px;
  }

  .domain-sidebar {
    max-height: 200px;
  }
}
</style>
