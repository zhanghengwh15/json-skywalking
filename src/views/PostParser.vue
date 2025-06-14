<template>
  <div class="post-parser">
    <div class="header">
      <h1>HTTP 请求解析工具</h1>
      <p>支持GET和POST请求，自动生成curl命令</p>
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
            <button @click="loadGetExample" class="example-btn">GET示例</button>
            <button @click="loadPostExample" class="example-btn">POST示例</button>
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
            <button @click="testRequest" :disabled="!curlCommand" class="test-btn">
              测试请求
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
    
    <div class="info-panel" v-if="parseInfo">
      <div class="info-grid">
        <div class="info-item">
          <h4>请求信息</h4>
          <div class="request-info">
            <div class="info-row">
              <span class="label">请求类型：</span>
              <span class="value method" :class="parseInfo.method.toLowerCase()">{{ parseInfo.method }}</span>
            </div>
            <div class="info-row">
              <span class="label">URL路径：</span>
              <span class="value">{{ parseInfo.urlPath }}</span>
            </div>
            <div class="info-row">
              <span class="label">完整URL：</span>
              <span class="value">{{ parseInfo.fullUrl }}</span>
            </div>
            <div class="info-row" v-if="parseInfo.queryParams">
              <span class="label">查询参数：</span>
              <span class="value">{{ parseInfo.queryParams.length }} 个</span>
            </div>
          </div>
        </div>
        <div class="info-item">
          <h4 v-if="parseInfo.method === 'POST'">JSON 数据</h4>
          <h4 v-else>查询参数</h4>
          <div class="json-preview" v-if="parseInfo.method === 'POST' && parseInfo.jsonData">
            <div class="json-stats">
              <div class="stat">
                <span>数据类型：</span>
                <span>{{ getJsonType(parseInfo.jsonData) }}</span>
              </div>
              <div class="stat">
                <span>字符长度：</span>
                <span>{{ parseInfo.jsonLength }}</span>
              </div>
              <div class="stat">
                <span>格式状态：</span>
                <span class="valid">✓ 有效JSON</span>
              </div>
            </div>
            <div class="json-content">
              <pre v-html="formatJsonWithHighlight(parseInfo.jsonData)"></pre>
            </div>
          </div>
          <div class="query-params" v-else-if="parseInfo.method === 'GET' && parseInfo.queryParams">
            <div class="params-stats">
              <div class="stat">
                <span>参数数量：</span>
                <span>{{ parseInfo.queryParams.length }}</span>
              </div>
            </div>
            <div class="params-list">
              <div v-for="param in parseInfo.queryParams" :key="param.key" class="param-item">
                <span class="param-key">{{ param.key }}:</span>
                <span class="param-value">{{ param.value }}</span>
              </div>
            </div>
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

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

// 提取URL路径（去掉:8080/之前的部分）
const extractUrlPath = (url: string): string => {
  // 处理完整URL，提取:8080/之后的部分
  const match = url.match(/:8080\/(.*)/)
  if (match && match[1]) {
    return match[1]
  }
  
  // 处理其他端口或localhost的情况
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
  return POST_URL_TEMPLATE
    .replace('{}', escapedJson)
    .replace('{}', urlPath)
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
      const parts = inputText.value.split('http.body:')
      
      if (parts.length !== 2) {
        throw new Error('POST请求格式不正确，应该包含 "http.body:" 分隔符')
      }
      
      const urlPart = parts[0].trim()
      const jsonPart = parts[1].trim()
      
      if (!urlPart) {
        throw new Error('URL不能为空')
      }
      
      if (!jsonPart) {
        throw new Error('JSON数据不能为空')
      }
      
      // 验证JSON格式
      if (!isValidJson(jsonPart)) {
        throw new Error('不是有效的JSON格式')
      }
      
      // 提取URL路径
      const urlPath = extractUrlPath(urlPart)
      
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
    alert('cURL命令已复制到剪贴板！')
  } catch (err) {
    // 降级方案
    const textArea = document.createElement('textarea')
    textArea.value = curlCommand.value
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
    alert('cURL命令已复制到剪贴板！')
  }
}

// 格式化curl命令
const formatCurl = () => {
  isFormatted.value = !isFormatted.value
}

// 测试请求
const testRequest = () => {
  if (!parseInfo.value) return
  
  const shouldExecute = confirm('是否要执行这个请求？请确保服务器正在运行。')
  if (!shouldExecute) return
  
  const headers: Record<string, string> = {
    'Accept-Language': 'zh-CN',
    'logLevel': 'debug'
  }
  
  if (parseInfo.value.method === 'POST') {
    headers['Content-Type'] = 'application/json'
  }
  
  const requestOptions: RequestInit = {
    method: parseInfo.value.method,
    headers
  }
  
  if (parseInfo.value.method === 'POST') {
    requestOptions.body = JSON.stringify(parseInfo.value.jsonData)
  }
  
  fetch(parseInfo.value.fullUrl, requestOptions)
  .then(response => {
    if (response.ok) {
      alert('请求成功！检查浏览器控制台查看响应。')
      console.log('Response:', response)
    } else {
      alert(`请求失败：${response.status} ${response.statusText}`)
    }
  })
  .catch(err => {
    alert(`请求错误：${err.message}`)
    console.error('Request error:', err)
  })
}

// 获取JSON数据类型
const getJsonType = (data: any): string => {
  if (Array.isArray(data)) return 'Array'
  if (data === null) return 'null'
  return typeof data === 'object' ? 'Object' : typeof data
}

// 格式化JSON显示（保留用于其他地方可能的使用）
// const formatJson = (data: any): string => {
//   return JSON.stringify(data, null, 2)
// }

// 格式化JSON并添加语法高亮
const formatJsonWithHighlight = (data: any): string => {
  let jsonString = JSON.stringify(data, null, 2)
  
  // 先处理键名（属性名）
  jsonString = jsonString.replace(/"([^"\\]*(\\.[^"\\]*)*)"\s*:/g, '<span class="json-key">"$1"</span><span class="json-punctuation">:</span>')
  
  // 处理字符串值
  jsonString = jsonString.replace(/:\s*"([^"\\]*(\\.[^"\\]*)*)"/g, ': <span class="json-string">"$1"</span>')
  
  // 处理数组中的字符串
  jsonString = jsonString.replace(/\[\s*"([^"\\]*(\\.[^"\\]*)*)"/g, '[<span class="json-string">"$1"</span>')
  jsonString = jsonString.replace(/,\s*"([^"\\]*(\\.[^"\\]*)*)"/g, ', <span class="json-string">"$1"</span>')
  
  // 处理数字
  jsonString = jsonString.replace(/:\s*(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/g, ': <span class="json-number">$1</span>')
  jsonString = jsonString.replace(/\[\s*(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/g, '[<span class="json-number">$1</span>')
  jsonString = jsonString.replace(/,\s*(-?\d+(?:\.\d+)?(?:[eE][+-]?\d+)?)/g, ', <span class="json-number">$1</span>')
  
  // 处理 null
  jsonString = jsonString.replace(/:\s*(null)/g, ': <span class="json-null">$1</span>')
  jsonString = jsonString.replace(/\[\s*(null)/g, '[<span class="json-null">$1</span>')
  jsonString = jsonString.replace(/,\s*(null)/g, ', <span class="json-null">$1</span>')
  
  // 处理 boolean
  jsonString = jsonString.replace(/:\s*(true|false)/g, ': <span class="json-boolean">$1</span>')
  jsonString = jsonString.replace(/\[\s*(true|false)/g, '[<span class="json-boolean">$1</span>')
  jsonString = jsonString.replace(/,\s*(true|false)/g, ', <span class="json-boolean">$1</span>')
  
  // 处理括号和逗号（排除已经处理过的）
  jsonString = jsonString.replace(/([{}[\]])/g, '<span class="json-punctuation">$1</span>')
  jsonString = jsonString.replace(/,(?![^<]*>)/g, '<span class="json-punctuation">,</span>')
  
  return jsonString
}

// 加载GET示例
const loadGetExample = () => {
  inputText.value = `http://192.168.100.31/poit-cloud-platform/areaEnergy/describeAreaElecStat?eid=9049ab5eff3c42e0a500ee53bc7e5360&operateUserId=500177&orgId=1000864&uid=427075f2e7ec43d1bd04ebd6d36af240&appVersion=1.0&dateType=day&areaId=7f8a123e49304dbdad5eabbb5229ef1e&statDate=2025-06&statDateEnd=&needLastPeriodRatio=1`
}

// 加载POST示例
const loadPostExample = () => {
  inputText.value = `http://localhost:8080/api/user/create
http.body: {
  "name": "张三",
  "email": "zhangsan@example.com",
  "age": 28,
  "department": "技术部",
  "skills": ["Java", "Python", "Vue"],
  "address": {
    "city": "北京",
    "district": "海淀区",
    "street": "中关村大街1号"
  },
  "isActive": true
}`
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
    alert('cURL 命令已复制到剪贴板！')
  } catch (err) {
    console.error('复制失败:', err)
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

// 组件挂载时加载历史记录
onMounted(() => {
  loadHistoryRecords()
})
</script>

<style scoped>
.post-parser {
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
  height: calc(100vh - 80px);
  display: flex;
  flex-direction: column;
}

.header {
  text-align: center;
  margin-bottom: 20px;
}

.header h1 {
  color: #2c3e50;
  margin-bottom: 10px;
}

.header p {
  color: #666;
  font-size: 14px;
}

.content {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  flex: 1;
  min-height: 0;
}

.input-section,
.output-section {
  display: flex;
  flex-direction: column;
  border: 1px solid #ddd;
  border-radius: 8px;
  overflow: hidden;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  background: linear-gradient(135deg, #3498db 0%, #2980b9 100%);
  color: white;
}

.section-header h3 {
  margin: 0;
  font-size: 16px;
}

.actions {
  display: flex;
  gap: 10px;
}

.parse-btn {
  background: #e67e22;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s ease;
}

.parse-btn:hover:not(:disabled) {
  background: #d35400;
  transform: translateY(-1px);
}

.parse-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.clear-btn,
.copy-btn,
.format-btn,
.example-btn,
.test-btn,
.history-btn {
  background: rgba(255, 255, 255, 0.2);
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.3);
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s ease;
}

.clear-btn:hover,
.copy-btn:hover:not(:disabled),
.format-btn:hover:not(:disabled),
.example-btn:hover,
.test-btn:hover:not(:disabled),
.history-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.test-btn {
  background: rgba(46, 204, 113, 0.8);
}

.test-btn:hover:not(:disabled) {
  background: rgba(39, 174, 96, 0.9);
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
  background: #f8f9fa;
}

.post-output {
  flex: 1;
  padding: 20px;
  background: #f8f9fa;
  overflow: auto;
}

.curl-display pre {
  font-family: 'Courier New', monospace;
  font-size: 13px;
  line-height: 1.4;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
  background: #2c3e50;
  color: #1abc9c;
  padding: 15px;
  border-radius: 4px;
  border-left: 4px solid #e67e22;
}

.placeholder {
  color: #999;
  text-align: center;
  padding: 40px 20px;
  font-style: italic;
}

.error-message {
  background: #fee;
  color: #c33;
  padding: 15px;
  border-radius: 4px;
  border-left: 4px solid #e74c3c;
}

.error-message h4 {
  margin: 0 0 10px 0;
  color: #e74c3c;
}

.error-message p {
  margin: 0;
  font-family: 'Courier New', monospace;
  font-size: 13px;
}

.info-panel {
  margin-top: 20px;
  padding: 20px;
  background: #f0f2f5;
  border-radius: 8px;
  border-left: 4px solid #3498db;
}

.info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 30px;
}

.info-item h4 {
  margin: 0 0 15px 0;
  color: #2c3e50;
}

.request-info {
  space-y: 8px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid #ddd;
}

.info-row:last-child {
  border-bottom: none;
}

.label {
  font-weight: 500;
  color: #666;
  min-width: 100px;
}

.value {
  color: #2c3e50;
  font-family: 'Courier New', monospace;
  font-size: 13px;
  word-break: break-all;
  text-align: right;
  flex: 1;
  margin-left: 10px;
}

.method {
  color: white;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: bold;
}

.method.post {
  background: #e67e22;
}

.method.get {
  background: #27ae60;
}

.json-preview {
  max-height: 300px;
  overflow: hidden;
}

.json-stats {
  margin-bottom: 15px;
}

.stat {
  display: flex;
  justify-content: space-between;
  padding: 4px 0;
  font-size: 13px;
}

.stat span:first-child {
  color: #666;
}

.stat span:last-child {
  color: #2c3e50;
  font-weight: 600;
}

.valid {
  color: #27ae60 !important;
}

.json-content {
  max-height: 200px;
  overflow-y: auto;
}

.json-content pre {
  font-family: 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.4;
  margin: 0;
  background: #1e1e1e;
  padding: 15px;
  border-radius: 4px;
  color: #d4d4d4;
  overflow-x: auto;
}

/* JSON 语法高亮样式 - 与VS Code Dark主题一致 */
.json-content .json-key {
  color: #9cdcfe; /* 浅蓝色 - 键名 */
}

.json-content .json-string {
  color: #ce9178; /* 橙黄色 - 字符串值 */
}

.json-content .json-number {
  color: #b5cea8; /* 浅绿色 - 数字 */
}

.json-content .json-boolean {
  color: #569cd6; /* 蓝色 - 布尔值 */
}

.json-content .json-null {
  color: #569cd6; /* 蓝色 - null值 */
}

.json-content .json-punctuation {
  color: #d4d4d4; /* 白色 - 标点符号 */
}

.query-params {
  max-height: 300px;
  overflow: hidden;
}

.params-stats {
  margin-bottom: 15px;
}

.params-list {
  max-height: 200px;
  overflow-y: auto;
  background: #ecf0f1;
  padding: 10px;
  border-radius: 4px;
}

.param-item {
  display: flex;
  padding: 4px 0;
  border-bottom: 1px solid #bdc3c7;
  font-family: 'Courier New', monospace;
  font-size: 12px;
}

.param-item:last-child {
  border-bottom: none;
}

.param-key {
  font-weight: bold;
  color: #2980b9;
  min-width: 120px;
  margin-right: 10px;
}

.param-value {
  color: #2c3e50;
  word-break: break-all;
  flex: 1;
}

/* 历史记录面板样式 */
.history-panel {
  margin-top: 20px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background: #fff;
  max-height: 600px;
  display: flex;
  flex-direction: column;
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  background: linear-gradient(135deg, #8e44ad 0%, #9b59b6 100%);
  color: white;
  border-radius: 8px 8px 0 0;
  flex-wrap: wrap;
  gap: 10px;
}

.history-stats {
  display: flex;
  gap: 15px;
  align-items: center;
}

.stat-item {
  font-size: 12px;
  padding: 3px 8px;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.2);
  border: 1px solid rgba(255, 255, 255, 0.3);
}

.stat-item.get {
  background: rgba(39, 174, 96, 0.3);
  border-color: rgba(39, 174, 96, 0.5);
}

.stat-item.post {
  background: rgba(230, 126, 34, 0.3);
  border-color: rgba(230, 126, 34, 0.5);
}

.stat-item.today {
  background: rgba(52, 152, 219, 0.3);
  border-color: rgba(52, 152, 219, 0.5);
}

.history-header h3 {
  margin: 0;
  font-size: 16px;
}

.history-actions {
  display: flex;
  gap: 10px;
}

.export-btn,
.batch-delete-btn,
.clear-history-btn,
.close-history-btn {
  background: rgba(255, 255, 255, 0.2);
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.3);
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.3s ease;
}

.export-btn:hover:not(:disabled),
.batch-delete-btn:hover:not(:disabled),
.clear-history-btn:hover:not(:disabled),
.close-history-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.export-btn:disabled,
.batch-delete-btn:disabled,
.clear-history-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.batch-delete-btn {
  background: rgba(231, 76, 60, 0.3);
  border-color: rgba(231, 76, 60, 0.5);
}

.batch-delete-btn:hover:not(:disabled) {
  background: rgba(231, 76, 60, 0.5);
}

/* 搜索和工具栏样式 */
.history-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 15px;
  background: #f8f9fa;
  border-bottom: 1px solid #e0e0e0;
}

.search-box {
  position: relative;
  flex: 1;
  max-width: 300px;
}

.search-input {
  width: 100%;
  padding: 8px 30px 8px 12px;
  border: 1px solid #ddd;
  border-radius: 20px;
  font-size: 14px;
  outline: none;
  transition: all 0.3s ease;
}

.search-input:focus {
  border-color: #3498db;
  box-shadow: 0 0 0 2px rgba(52, 152, 219, 0.1);
}

.search-icon {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: #666;
  pointer-events: none;
}

.toolbar-actions {
  display: flex;
  gap: 10px;
}

.select-all-btn {
  background: #3498db;
  color: white;
  border: none;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.3s ease;
}

.select-all-btn:hover {
  background: #2980b9;
}

.history-content {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.history-loading,
.history-empty {
  text-align: center;
  padding: 40px 20px;
  color: #666;
  font-style: italic;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.history-item {
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  padding: 12px;
  cursor: pointer;
  transition: all 0.3s ease;
  background: #fafafa;
  position: relative;
}

.history-item:hover {
  border-color: #3498db;
  background: #f0f8ff;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(52, 152, 219, 0.1);
}

.history-item.selected {
  border-color: #3498db;
  background: #e8f4fd;
  box-shadow: 0 0 0 2px rgba(52, 152, 219, 0.1);
}

.history-item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.history-item-left {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
}

.record-checkbox {
  margin: 0;
  transform: scale(1.1);
  cursor: pointer;
}

.history-item-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.history-method {
  padding: 2px 6px;
  border-radius: 10px;
  font-size: 10px;
  font-weight: bold;
  color: white;
}

.history-method.get {
  background: #27ae60;
}

.history-method.post {
  background: #e67e22;
}

.history-title {
  font-weight: 600;
  color: #2c3e50;
}

.history-item-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.history-time {
  font-size: 12px;
  color: #666;
}

.copy-curl-btn,
.delete-record-btn {
  border: none;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  cursor: pointer;
  font-size: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s ease;
}

.copy-curl-btn {
  background: #3498db;
  color: white;
}

.copy-curl-btn:hover {
  background: #2980b9;
  transform: scale(1.1);
}

.delete-record-btn {
  background: #e74c3c;
  color: white;
}

.delete-record-btn:hover {
  background: #c0392b;
  transform: scale(1.1);
}

.history-item-url {
  font-family: 'Courier New', monospace;
  font-size: 12px;
  color: #2980b9;
  margin-bottom: 4px;
  word-break: break-all;
}

.history-item-curl {
  font-family: 'Courier New', monospace;
  font-size: 11px;
  color: #666;
  background: #ecf0f1;
  padding: 4px 8px;
  border-radius: 3px;
  word-break: break-all;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .content {
    grid-template-columns: 1fr;
    grid-template-rows: 1fr 1fr;
  }
  
  .info-grid {
    grid-template-columns: 1fr;
    gap: 20px;
  }
  
  .post-parser {
    padding: 10px;
  }
  
  .actions {
    flex-wrap: wrap;
    gap: 5px;
  }
  
  .section-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }
  
  .info-row {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
  
  .value {
    text-align: left;
    margin-left: 0;
  }
  
  .history-header {
    flex-direction: column;
    gap: 10px;
  }
  
  .history-stats {
    justify-content: center;
    flex-wrap: wrap;
  }
  
  .history-actions {
    justify-content: center;
    flex-wrap: wrap;
  }
  
  .history-toolbar {
    flex-direction: column;
    gap: 10px;
    align-items: stretch;
  }
  
  .search-box {
    max-width: none;
  }
  
  .toolbar-actions {
    justify-content: center;
  }
  
  .history-item-left {
    align-items: flex-start;
  }
  
  .history-item-actions {
    flex-direction: column;
    gap: 5px;
  }
  
  .copy-curl-btn,
  .delete-record-btn {
    width: 30px;
    height: 30px;
  }
}
</style> 