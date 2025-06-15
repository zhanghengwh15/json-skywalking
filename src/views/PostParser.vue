<template>
  <div class="post-parser">
    <div class="header">
      <h1>HTTP 请求解析工具</h1>
      <p>支持HTTP请求解析，使用Ctrl+V快速生成curl命令</p>
    </div>
    
    <div class="content">
      <div class="input-section">
        <div class="section-header">
          <h3>输入请求信息</h3>
          <div class="actions">
            <button @click="parseRequest" :disabled="loading" class="parse-btn">
              {{ loading ? '解析中...' : '生成 cURL' }}
            </button>
            <button @click="clearInput" class="clear-btn">清空</button>
            <button @click="toggleHistory" class="history-btn">
              {{ showHistory ? '隐藏历史' : '查看历史' }}
            </button>
          </div>
        </div>
        <textarea
          v-model="inputText"
          placeholder="GET请求格式：直接输入完整URL&#10;POST请求格式：URL&#10;http.body: JSON数据"
          class="post-input"
          @input="resetError"
        ></textarea>
      </div>
      
      <div class="output-section">
        <div class="section-header">
          <h3>生成的 cURL 命令</h3>
          <div class="actions">
            <button @click="copyToClipboard" :disabled="!curlCommand" class="copy-btn">
              复制命令
            </button>
            <button @click="formatCurl" :disabled="!curlCommand" class="format-btn">
              {{ isFormatted ? '压缩' : '格式化' }}
            </button>
            <button @click="copyUrlPath" :disabled="!parseInfo" class="copy-path-btn">
              复制路径
            </button>
            <button 
              v-if="parseInfo && parseInfo.method === 'POST' && parseInfo.jsonData" 
              @click="viewJson" 
              class="view-json-btn"
            >
              查看JSON
            </button>
          </div>
        </div>
        <div class="post-output">
          <div v-if="error" class="error-message">
            <h4>解析错误：</h4>
            <p>{{ error }}</p>
          </div>
          <div v-else-if="curlCommand" class="curl-display">
            <pre>{{ formattedCurl }}</pre>
          </div>
          <div v-else class="placeholder">
            生成的cURL命令将在这里显示...
          </div>
        </div>
      </div>
    </div>
    
    <!-- 历史记录面板 -->
    <div class="history-panel" v-if="showHistory">
      <div class="history-header">
        <h3>解析历史记录</h3>
        <div class="history-stats">
          <span class="stat-item">总计: {{ historyStats.total }}</span>
          <span class="stat-item get">GET: {{ historyStats.getCount }}</span>
          <span class="stat-item post">POST: {{ historyStats.postCount }}</span>
          <span class="stat-item today">今日: {{ historyStats.todayCount }}</span>
        </div>
        <div class="history-actions">
          <button @click="exportHistory" :disabled="historyRecords.length === 0" class="export-btn">
            导出
          </button>
          <button @click="deleteSelectedRecords" :disabled="selectedRecords.size === 0" class="batch-delete-btn">
            删除选中 ({{ selectedRecords.size }})
          </button>
          <button @click="clearHistoryRecords" :disabled="historyRecords.length === 0" class="clear-history-btn">
            清空历史
          </button>
          <button @click="showHistory = false" class="close-history-btn">
            关闭
          </button>
        </div>
      </div>
      
      <!-- 搜索和操作栏 -->
      <div class="history-toolbar">
        <div class="search-box">
          <input 
            type="text" 
            v-model="historySearchQuery" 
            placeholder="搜索历史记录..." 
            class="search-input"
          />
          <span class="search-icon">🔍</span>
        </div>
        <div class="toolbar-actions">
          <button @click="toggleSelectAll" class="select-all-btn">
            {{ selectedRecords.size === filteredHistoryRecords.length && filteredHistoryRecords.length > 0 ? '取消全选' : '全选' }}
          </button>
        </div>
      </div>
      
      <div class="history-content">
        <div v-if="historyLoading" class="history-loading">
          加载中...
        </div>
        <div v-else-if="historyRecords.length === 0" class="history-empty">
          暂无历史记录
        </div>
        <div v-else class="history-list">
          <div 
            v-for="record in filteredHistoryRecords" 
            :key="record.id" 
            class="history-item"
            :class="{ 'selected': selectedRecords.has(record.id) }"
            @click="loadFromHistory(record)"
          >
            <div class="history-item-header">
              <div class="history-item-left">
                <input 
                  type="checkbox" 
                  :checked="selectedRecords.has(record.id)"
                  @click.stop="toggleRecordSelection(record.id)"
                  class="record-checkbox"
                />
              <div class="history-item-title">
                <span class="history-method" :class="record.request_type.toLowerCase()">
                  {{ record.request_type }}
                </span>
                <span class="history-title">{{ record.title }}</span>
                </div>
              </div>
              <div class="history-item-actions">
                <span class="history-time" :title="formatTime(record.timestamp)">
                  {{ formatRelativeTime(record.timestamp) }}
                </span>
                <button 
                  @click.stop="copyRecordCurl(record)" 
                  class="copy-curl-btn"
                  title="复制 cURL 命令"
                >
                  📋
                </button>
                <button 
                  @click.stop="deleteHistoryRecord(record.id)" 
                  class="delete-record-btn"
                  title="删除记录"
                >
                  ✕
                </button>
              </div>
            </div>
            <div class="history-item-url">{{ record.url }}</div>
            <div class="history-item-curl">{{ record.curl_command.substring(0, 100) }}...</div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 提示消息 Toast -->
    <div v-if="toastMessage" class="toast-container">
      <div class="toast" :class="toastMessage.type">
        {{ toastMessage.message }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import '../styles/post-parser.css'

// 定义类型
interface ParseRecord {
  id: string
  timestamp: number
  request_type: string
  url: string
  json_data?: any
  curl_command: string
  title?: string
}

// 获取router实例
const router = useRouter()

// 响应式数据
const inputText = ref('')
const curlCommand = ref('')
const error = ref('')
const loading = ref(false)
const isFormatted = ref(false)
const parseInfo = ref<any>(null)
const showHistory = ref(false)
const historyRecords = ref<ParseRecord[]>([])
const historyLoading = ref(false)
const historySearchQuery = ref('')
const selectedRecords = ref<Set<string>>(new Set())

// 提示消息状态
const toastMessage = ref<{type: 'success' | 'error' | 'info', message: string} | null>(null)

// curl模板
const POST_URL_TEMPLATE = 'curl -X POST -H "Accept-Language:zh-CN" -H "logLevel:debug" -H "Content-Type:application/json" -d \'{}\' --url "http://localhost:8080/{}"'
const GET_URL_TEMPLATE = 'curl -X GET -H "Accept-Language:zh-CN" -H "logLevel:debug" --url "http://localhost:8080/{}"'

// 格式化后的curl命令
const formattedCurl = computed(() => {
  if (!curlCommand.value) return ''
  
  if (isFormatted.value) {
    if (curlCommand.value.includes('-X POST')) {
      return curlCommand.value
        .replace(/curl/, 'curl \\\n ')
        .replace(/-X POST/, '-X POST \\\n ')
        .replace(/-H "Accept-Language:zh-CN"/, '-H "Accept-Language:zh-CN" \\\n ')
        .replace(/-H "logLevel:debug"/, '-H "logLevel:debug" \\\n ')
        .replace(/-H "Content-Type:application\/json"/, '-H "Content-Type:application/json" \\\n ')
        .replace(/-d '/, '-d \'\\\n')
        .replace(/' --url/, '\' \\\n --url')
    } else {
      return curlCommand.value
        .replace(/curl/, 'curl \\\n ')
        .replace(/-X GET/, '-X GET \\\n ')
        .replace(/-H "Accept-Language:zh-CN"/, '-H "Accept-Language:zh-CN" \\\n ')
        .replace(/-H "logLevel:debug"/, '-H "logLevel:debug" \\\n ')
        .replace(/--url/, '--url')
    }
  }
  
  return curlCommand.value
})

// 验证JSON格式
const isValidJson = (str: string): boolean => {
  try {
    JSON.parse(str)
    return true
  } catch (e) {
    return false
  }
}

// 智能提取有效的JSON部分（去掉后面的额外内容）
const extractValidJson = (text: string): string => {
  text = text.trim()
  
  // 如果不是以 { 或 [ 开始，寻找第一个 { 或 [
  let startIndex = 0
  for (let i = 0; i < text.length; i++) {
    if (text[i] === '{' || text[i] === '[') {
      startIndex = i
      break
    }
  }
  
  if (startIndex === text.length) {
    return text // 没找到JSON开始符号，返回原文本
  }
  
  // 使用栈来匹配括号，找到完整JSON的结束位置
  const stack: string[] = []
  let inString = false
  let escaped = false
  
  for (let i = startIndex; i < text.length; i++) {
    const char = text[i]
    
    if (inString) {
      if (escaped) {
        escaped = false
      } else if (char === '\\') {
        escaped = true
      } else if (char === '"') {
        inString = false
      }
    } else {
      if (char === '"') {
        inString = true
      } else if (char === '{') {
        stack.push('}')
      } else if (char === '[') {
        stack.push(']')
      } else if (char === '}' || char === ']') {
        if (stack.length === 0 || stack.pop() !== char) {
          // 括号不匹配，继续寻找
          continue
        }
        
        // 如果栈为空，说明找到了完整的JSON
        if (stack.length === 0) {
          return text.substring(startIndex, i + 1)
        }
      }
    }
  }
  
  // 如果没找到完整的JSON结束，尝试验证整个字符串
  const fullJson = text.substring(startIndex)
  if (isValidJson(fullJson)) {
    return fullJson
  }
  
  // 最后尝试按行分割，找到第一个有效的JSON
  const lines = text.split('\n')
  let jsonCandidate = ''
  
  for (const line of lines) {
    const trimmedLine = line.trim()
    if (!trimmedLine || trimmedLine.startsWith('status_code:') || trimmedLine.startsWith('error:')) {
      continue
    }
    
    jsonCandidate += (jsonCandidate ? ' ' : '') + trimmedLine
    
    if (isValidJson(jsonCandidate)) {
      return jsonCandidate
    }
  }
  
  return text // 如果所有方法都失败，返回原文本
}

// 提取URL路径（去掉协议、域名/IP、端口之前的部分）
const extractUrlPath = (url: string): string => {
  // 处理完整URL（包含协议）
  if (url.includes('://')) {
    try {
      const urlObj = new URL(url)
      // 去掉开头的斜杠
      return urlObj.pathname.replace(/^\/+/, '') + (urlObj.search || '')
    } catch (e) {
      // 如果URL构造失败，使用正则表达式提取
      const match = url.match(/^https?:\/\/[^\/]+\/(.*)/)
  if (match && match[1]) {
    return match[1]
      }
    }
  }
  
  // 处理特定端口的情况
  const portMatch = url.match(/:(\d+)\/(.*)/)
  if (portMatch && portMatch[2]) {
    return portMatch[2]
  }
  
  // 处理localhost的情况
  if (url.includes('localhost/')) {
    return url.split('localhost/')[1] || url
  }
  
  // 处理IP地址的情况，如 192.168.100.31/path
  const ipMatch = url.match(/\/\/[\d.]+\/(.*)/)
  if (ipMatch && ipMatch[1]) {
    return ipMatch[1]
  }
  
  // 处理域名的情况
  if (url.includes('http://') || url.includes('https://')) {
    const urlParts = url.split('/')
    return urlParts.slice(3).join('/') // 去掉协议和域名部分
  }
  
  // 如果都没匹配到，直接返回去掉开头斜杠的URL
  return url.replace(/^\/+/, '')
}

// 解析查询参数
const parseQueryParams = (url: string): Array<{key: string, value: string}> => {
  const queryString = url.includes('?') ? url.split('?')[1] : ''
  if (!queryString) return []
  
  return queryString.split('&').map(param => {
    const [key, value] = param.split('=')
    return {
      key: decodeURIComponent(key || ''),
      value: decodeURIComponent(value || '')
    }
  })
}

// 生成curl命令
const generateGetCurl = (urlPath: string): string => {
  return GET_URL_TEMPLATE.replace('{}', urlPath)
}

const generatePostCurl = (jsonData: string, urlPath: string): string => {
  // 转义JSON中的单引号
  const escapedJson = jsonData.replace(/'/g, "'\"'\"'")
  // 使用更精确的替换，避免多个{}替换顺序问题
  return POST_URL_TEMPLATE
    .replace("'{}'", `'${escapedJson}'`)
    .replace("http://localhost:8080/{}", `http://localhost:8080/${urlPath}`)
}

// 判断是否为POST请求（包含http.body:）
const isPostRequest = (text: string): boolean => {
  return text.includes('http.body:')
}

// 主要的请求解析函数
const parseRequest = async () => {
  if (!inputText.value.trim()) {
    error.value = '请输入URL或请求信息'
    return
  }
  
  loading.value = true
  error.value = ''
  curlCommand.value = ''
  parseInfo.value = null
  
  try {
    // 模拟异步处理
    await new Promise(resolve => setTimeout(resolve, 100))
    
    const isPost = isPostRequest(inputText.value)
    
    if (isPost) {
      // POST请求处理
      const bodyIndex = inputText.value.indexOf('http.body:')
      
      if (bodyIndex === -1) {
        throw new Error('POST请求格式不正确，应该包含 "http.body:" 分隔符')
      }
      
      const urlPart = inputText.value.substring(0, bodyIndex).trim()
      let jsonPart = inputText.value.substring(bodyIndex + 'http.body:'.length).trim()
      
      if (!urlPart) {
        throw new Error('URL不能为空')
      }
      
      if (!jsonPart) {
        throw new Error('JSON数据不能为空')
      }
      
      // 智能提取有效的JSON部分（去掉后面的额外内容如status_code等）
      jsonPart = extractValidJson(jsonPart)
      
      // 验证JSON格式
      if (!isValidJson(jsonPart)) {
        throw new Error('不是有效的JSON格式')
      }
      
      // 提取URL路径
      const urlPath = extractUrlPath(urlPart)
      
      // 调试信息
      console.log('POST请求解析结果:', {
        原始输入长度: inputText.value.length,
        URL部分: urlPart,
        JSON部分长度: jsonPart.length,
        JSON开头: jsonPart.substring(0, 100) + (jsonPart.length > 100 ? '...' : ''),
        提取的路径: urlPath,
        完整URL: `http://localhost:8080/${urlPath}`
      })
      
      // 生成curl命令
      const curl = generatePostCurl(jsonPart, urlPath)
      
      curlCommand.value = curl
      parseInfo.value = {
        method: 'POST',
        urlPath,
        fullUrl: `http://localhost:8080/${urlPath}`,
        jsonData: JSON.parse(jsonPart),
        jsonLength: jsonPart.length,
        originalUrl: urlPart
      }
      
      // 自动保存记录
      await saveParseRecord()
      
    } else {
      // GET请求处理
      const urlPart = inputText.value.trim()
      
      if (!urlPart) {
        throw new Error('URL不能为空')
      }
      
      // 提取URL路径
      const urlPath = extractUrlPath(urlPart)
      
      // 解析查询参数
      const queryParams = parseQueryParams(urlPath)
      
      // 生成curl命令
      const curl = generateGetCurl(urlPath)
      
      curlCommand.value = curl
      parseInfo.value = {
        method: 'GET',
        urlPath,
        fullUrl: `http://localhost:8080/${urlPath}`,
        queryParams,
        originalUrl: urlPart
      }
      
      // 自动保存记录
      await saveParseRecord()
    }
    
  } catch (err) {
    error.value = err instanceof Error ? err.message : '解析失败'
  } finally {
    loading.value = false
  }
}

// 清空输入
const clearInput = () => {
  inputText.value = ''
  curlCommand.value = ''
  error.value = ''
  parseInfo.value = null
  isFormatted.value = false
}

// 重置错误
const resetError = () => {
  error.value = ''
}

// 复制到剪贴板
const copyToClipboard = async () => {
  if (!curlCommand.value) return
  
  try {
    await navigator.clipboard.writeText(curlCommand.value)
    showToast('success', '✅ cURL命令已复制！')
  } catch (err) {
    // 降级方案
    try {
    const textArea = document.createElement('textarea')
    textArea.value = curlCommand.value
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
      showToast('success', '✅ cURL命令已复制！')
    } catch (fallbackErr) {
      showToast('error', '❌ 复制失败，请稍后重试')
    }
  }
}

// 格式化curl命令
const formatCurl = () => {
  isFormatted.value = !isFormatted.value
}

// 显示提示消息
const showToast = (type: 'success' | 'error' | 'info', message: string, duration: number = 1000) => {
  toastMessage.value = { type, message }
  setTimeout(() => {
    toastMessage.value = null
  }, duration)
}

// 复制URL路径
const copyUrlPath = async () => {
  if (!parseInfo.value) return
  
  try {
    await navigator.clipboard.writeText(parseInfo.value.urlPath)
    showToast('success', '✅ URL路径已复制！')
  } catch (err) {
    // 降级方案
    try {
      const textArea = document.createElement('textarea')
      textArea.value = parseInfo.value.urlPath
      document.body.appendChild(textArea)
      textArea.select()
      document.execCommand('copy')
      document.body.removeChild(textArea)
      showToast('success', '✅ URL路径已复制！')
    } catch (fallbackErr) {
      showToast('error', '❌ 复制失败，请稍后重试')
    }
  }
}

// 保存当前状态到localStorage
const saveCurrentState = () => {
  if (parseInfo.value && curlCommand.value) {
    const currentState = {
      inputText: inputText.value,
      curlCommand: curlCommand.value,
      parseInfo: parseInfo.value,
      isFormatted: isFormatted.value,
      timestamp: Date.now()
    }
    localStorage.setItem('postParserState', JSON.stringify(currentState))
    console.log('PostParser: 状态已保存')
  }
}

// 恢复状态从localStorage
const restoreState = () => {
  try {
    const savedState = localStorage.getItem('postParserState')
    if (savedState) {
      const state = JSON.parse(savedState)
      
      // 检查状态是否过期（30分钟）
      const now = Date.now()
      const thirtyMinutes = 30 * 60 * 1000
      
      if (now - state.timestamp < thirtyMinutes) {
        inputText.value = state.inputText || ''
        curlCommand.value = state.curlCommand || ''
        parseInfo.value = state.parseInfo || null
        isFormatted.value = state.isFormatted || false
        error.value = ''
        
        console.log('PostParser: 状态已恢复')
        showToast('info', '✨ 已恢复之前的解析状态', 2000)
    } else {
        // 状态过期，清除
        localStorage.removeItem('postParserState')
        console.log('PostParser: 状态已过期，已清除')
      }
    }
  } catch (err) {
    console.error('PostParser: 恢复状态失败:', err)
    localStorage.removeItem('postParserState')
  }
}

// 查看JSON（跳转到JsonParser页面）
const viewJson = async () => {
  if (!parseInfo.value || parseInfo.value.method !== 'POST' || !parseInfo.value.jsonData) return
  
  try {
    // 保存当前状态
    saveCurrentState()
    
    // 将JSON数据存储到localStorage，然后跳转到JsonParser页面
    const jsonString = JSON.stringify(parseInfo.value.jsonData, null, 2)
    localStorage.setItem('tempJsonData', jsonString)
    
    console.log('准备跳转到JsonParser页面，JSON数据已保存到localStorage')
    
    // 使用Vue Router进行页面跳转
    await router.push('/json-parser')
    
    console.log('跳转成功')
  } catch (err) {
    console.error('查看JSON失败:', err)
    showToast('error', '❌ 跳转失败，请稍后重试')
    
    // 降级方案：直接修改URL
    try {
      window.location.href = '/json-parser'
    } catch (fallbackErr) {
      console.error('降级跳转也失败:', fallbackErr)
    }
  }
}

// 生成唯一ID
const generateId = (): string => {
  return Date.now().toString(36) + Math.random().toString(36).substr(2)
}

// 保存解析记录
const saveParseRecord = async () => {
  if (!parseInfo.value || !curlCommand.value) return
  
  try {
    const record: ParseRecord = {
      id: generateId(),
      timestamp: Date.now(),
      request_type: parseInfo.value.method,
      url: parseInfo.value.originalUrl,
      json_data: parseInfo.value.jsonData || null,
      curl_command: curlCommand.value,
      title: `${parseInfo.value.method} - ${parseInfo.value.urlPath.split('/').pop() || 'API'}`
    }
    
    await invoke('save_parse_record', { record })
    await loadHistoryRecords()
  } catch (err) {
    console.error('保存记录失败:', err)
  }
}

// 加载历史记录
const loadHistoryRecords = async () => {
  try {
    historyLoading.value = true
    const records = await invoke('get_parse_records') as ParseRecord[]
    historyRecords.value = records
  } catch (err) {
    console.error('加载历史记录失败:', err)
  } finally {
    historyLoading.value = false
  }
}

// 删除历史记录
const deleteHistoryRecord = async (recordId: string) => {
  try {
    await invoke('delete_parse_record', { recordId })
    await loadHistoryRecords()
  } catch (err) {
    console.error('删除记录失败:', err)
  }
}

// 清空历史记录
const clearHistoryRecords = async () => {
  if (!confirm('确定要清空所有历史记录吗？')) return
  
  try {
    await invoke('clear_parse_records')
    await loadHistoryRecords()
  } catch (err) {
    console.error('清空记录失败:', err)
  }
}

// 从历史记录加载
const loadFromHistory = (record: ParseRecord) => {
  if (record.request_type === 'POST' && record.json_data) {
    inputText.value = `${record.url}\nhttp.body: ${JSON.stringify(record.json_data, null, 2)}`
  } else {
    inputText.value = record.url
  }
  showHistory.value = false
}

// 切换历史记录显示
const toggleHistory = async () => {
  showHistory.value = !showHistory.value
  if (showHistory.value && historyRecords.value.length === 0) {
    await loadHistoryRecords()
  }
}

// 格式化时间
const formatTime = (timestamp: number): string => {
  return new Date(timestamp).toLocaleString('zh-CN')
}

// 相对时间格式化
const formatRelativeTime = (timestamp: number): string => {
  const now = Date.now()
  const diff = now - timestamp
  
  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`
  if (diff < 2592000000) return `${Math.floor(diff / 86400000)} 天前`
  
  return formatTime(timestamp)
}

// 导出历史记录
const exportHistory = () => {
  const exportData = {
    exportTime: new Date().toISOString(),
    totalRecords: historyRecords.value.length,
    records: historyRecords.value
  }
  
  const dataStr = JSON.stringify(exportData, null, 2)
  const dataBlob = new Blob([dataStr], { type: 'application/json' })
  const url = URL.createObjectURL(dataBlob)
  
  const link = document.createElement('a')
  link.href = url
  link.download = `http-requests-history-${new Date().toISOString().split('T')[0]}.json`
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  URL.revokeObjectURL(url)
}

// 批量删除选中记录
const deleteSelectedRecords = async () => {
  if (selectedRecords.value.size === 0) return
  
  const count = selectedRecords.value.size
  if (!confirm(`确定要删除选中的 ${count} 条记录吗？`)) return
  
  try {
    for (const recordId of selectedRecords.value) {
      await invoke('delete_parse_record', { recordId })
    }
    selectedRecords.value.clear()
    await loadHistoryRecords()
  } catch (err) {
    console.error('批量删除记录失败:', err)
  }
}

// 切换记录选择状态
const toggleRecordSelection = (recordId: string) => {
  if (selectedRecords.value.has(recordId)) {
    selectedRecords.value.delete(recordId)
  } else {
    selectedRecords.value.add(recordId)
  }
}

// 全选/全不选
const toggleSelectAll = () => {
  if (selectedRecords.value.size === filteredHistoryRecords.value.length) {
    selectedRecords.value.clear()
  } else {
    selectedRecords.value.clear()
    filteredHistoryRecords.value.forEach(record => {
      selectedRecords.value.add(record.id)
    })
  }
}

// 复制记录的 cURL 命令
const copyRecordCurl = async (record: ParseRecord) => {
  try {
    await navigator.clipboard.writeText(record.curl_command)
    showToast('success', '✅ cURL 命令已复制！')
  } catch (err) {
    console.error('复制失败:', err)
    showToast('error', '❌ 复制失败，请稍后重试')
  }
}

// 过滤后的历史记录
const filteredHistoryRecords = computed(() => {
  if (!historySearchQuery.value.trim()) return historyRecords.value
  
  const query = historySearchQuery.value.toLowerCase()
  return historyRecords.value.filter(record => 
    record.url.toLowerCase().includes(query) ||
    record.request_type.toLowerCase().includes(query) ||
    (record.title && record.title.toLowerCase().includes(query)) ||
    record.curl_command.toLowerCase().includes(query)
  )
})

// 统计信息
const historyStats = computed(() => {
  const total = historyRecords.value.length
  const getCount = historyRecords.value.filter(r => r.request_type === 'GET').length
  const postCount = historyRecords.value.filter(r => r.request_type === 'POST').length
  const today = new Date()
  const todayStart = new Date(today.getFullYear(), today.getMonth(), today.getDate()).getTime()
  const todayCount = historyRecords.value.filter(r => r.timestamp >= todayStart).length
  
  return { total, getCount, postCount, todayCount }
})

// 键盘事件处理器
const handleKeydown = async (event: KeyboardEvent) => {
  const isMac = /Mac|iPhone|iPod|iPad/i.test(navigator.userAgent)
  const isModifierPressed = isMac ? event.metaKey : event.ctrlKey
  
  // Ctrl+V 或 Command+V 直接读取剪贴板并格式化
  if (isModifierPressed && event.key.toLowerCase() === 'v') {
    // 阻止默认粘贴行为
    event.preventDefault()
    
    try {
      // 读取剪贴板内容
      const clipboardText = await navigator.clipboard.readText()
      
      if (clipboardText.trim()) {
        // 将内容写入输入框
        inputText.value = clipboardText.trim()
        
        // 自动开始解析格式化
        setTimeout(() => {
          parseRequest()
        }, 50)
      }
    } catch (error) {
      console.error('读取剪贴板失败:', error)
      // 如果读取剪贴板失败，提示用户手动粘贴
      showToast('error', '❌ 无法读取剪贴板，请手动粘贴后点击生成 cURL 按钮', 3000)
    }
  }
}

// 组件挂载时加载历史记录
onMounted(() => {
  // 恢复之前的状态
  restoreState()
  // 加载历史记录
  loadHistoryRecords()
  // 添加键盘事件监听器
  document.addEventListener('keydown', handleKeydown)
})

// 组件卸载时移除事件监听器
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.post-parser {
  flex: 1;
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #1e1e1e;
  color: #d4d4d4;
  overflow: hidden;
  box-sizing: border-box;
  width: 100%;
  max-width: 1400px;
  margin: 0 auto;
}

.header {
  text-align: center;
  margin-bottom: 20px;
  flex-shrink: 0;
  padding: 0 20px;
}

.content {
  flex: 1;
  min-height: 0;
  height: 100%;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  overflow: hidden;
  box-sizing: border-box;
}

.input-section,
.output-section {
  flex: 1;
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  border: 1px solid #3c3c3c;
  border-radius: 8px;
  overflow: hidden;
  background: #252526;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  background: #2d2d30;
  color: #cccccc;
  border-bottom: 1px solid #3c3c3c;
  flex-shrink: 0;
}

.post-input {
  flex: 1;
  padding: 20px;
  border: none;
  outline: none;
  font-family: 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.5;
  resize: none;
  background: #1e1e1e;
  color: #d4d4d4;
  overflow: auto;
  min-height: 0;
}

.post-output {
  flex: 1;
  padding: 20px;
  background: #1e1e1e;
  overflow: auto;
  color: #d4d4d4;
  min-height: 0;
}
</style> 