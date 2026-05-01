<template>
  <div class="cookie-bridge">
    <div class="sidebar">
      <h3>域名列表</h3>
      <ul v-if="domains.length > 0">
        <li
          v-for="d in domains"
          :key="d"
          :class="{ active: d === selectedDomain }"
          @click="selectDomain(d)"
        >
          {{ d }}
        </li>
      </ul>
      <div v-else class="empty">
        等待 Chrome 扩展推送数据
      </div>
    </div>
    <div class="detail">
      <div v-if="selectedDomain">
        <h4>Cookies</h4>
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
              <td>{{ c.name }}</td>
              <td>{{ c.value }}</td>
              <td>{{ c.path }}</td>
              <td>{{ c.expires }}</td>
              <td>{{ c.secure }}</td>
              <td>{{ c.httpOnly }}</td>
            </tr>
          </tbody>
        </table>
        <p v-else class="empty-table">无 cookies</p>

        <h4>Local Storage</h4>
        <table v-if="currentDomain.localStorage.length > 0">
          <thead>
            <tr>
              <th>Key</th>
              <th>Value</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="ls in currentDomain.localStorage" :key="ls.key">
              <td>{{ ls.key }}</td>
              <td>{{ ls.value }}</td>
            </tr>
          </tbody>
        </table>
        <p v-else class="empty-table">无 localStorage</p>
      </div>
      <div v-else class="empty">
        请从左侧选择一个域名
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

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
  height: 100vh;
}
.sidebar {
  width: 240px;
  border-right: 1px solid #ddd;
  padding: 16px;
  overflow-y: auto;
}
.sidebar h3 {
  margin-bottom: 12px;
}
.sidebar ul {
  list-style: none;
  padding: 0;
}
.sidebar li {
  padding: 8px 12px;
  cursor: pointer;
  border-radius: 4px;
}
.sidebar li:hover {
  background: #f0f0f0;
}
.sidebar li.active {
  background: #e0e0e0;
  font-weight: bold;
}
.empty {
  color: #999;
  padding: 20px;
  text-align: center;
}
.detail {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}
.detail h4 {
  margin-top: 16px;
  margin-bottom: 8px;
}
table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}
th, td {
  border: 1px solid #ddd;
  padding: 6px 8px;
  text-align: left;
}
th {
  background: #f5f5f5;
}
.empty-table {
  color: #999;
  padding: 12px;
}
</style>
