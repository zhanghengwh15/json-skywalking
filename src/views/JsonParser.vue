<template>
  <div class="json-parser">
    <div class="header">
      <h1>JSON 解析与格式化工具</h1>
      <p>使用快捷键或按钮从剪贴板加载JSON数据</p>
    </div>
    
    <div class="content">
      <div class="input-section">
        <div class="section-header">
          <h3>JSON 结构视图</h3>
          <div class="actions">
            <button @click="getClipboardContent" class="clipboard-btn" :title="'获取剪贴板 (全局快捷键: ⌘⇧G)'">
              获取剪贴板
              <span class="shortcut-hint">⌘⇧G</span>
              <span class="global-indicator">🌍</span>
            </button>
          </div>
        </div>
        <div class="json-tree-container">
          <div v-if="error" class="error-message">
            <h4>解析错误：</h4>
            <p>{{ error }}</p>
          </div>
          <div v-else-if="parsedJson" class="json-tree">
            <JsonTreeNode 
              :data="parsedJson" 
              :key-name="'root'" 
              :is-root="true"
              :level="0"
              :selected-path="selectedKeyPath"
              :current-path="[]"
              @select="onSelectKey"
              @contextmenu="onContextMenu"
            />
          </div>
          <div v-else class="placeholder">
            <div class="placeholder-icon">📋</div>
            <p>使用 <kbd>⌘⇧G</kbd> 或 <kbd>⌘V</kbd> 从剪贴板加载JSON数据</p>
            <p class="placeholder-hint">或点击上方的"获取剪贴板"按钮</p>
          </div>
        </div>
        <div v-if="clipboardStatus" class="clipboard-status" :class="clipboardStatus.type">
          {{ clipboardStatus.message }}
        </div>
      </div>
      
      <div class="output-section">
        <div class="section-header">
          <h3>值内容</h3>
          <div class="actions">
            <button @click="copySelectedValue" :disabled="!parsedJson" class="copy-btn">
              复制值
            </button>
            <button @click="toggleHistory" class="history-btn">
              历史记录 ({{ historyList.length }})
            </button>
          </div>
        </div>
        <div class="json-output">
          <div v-if="error" class="error-message">
            <h4>解析错误：</h4>
            <p>{{ error }}</p>
          </div>
          <div v-else-if="!parsedJson" class="placeholder">
            解析后的JSON将在这里显示...
          </div>
          <div v-else class="json-display" v-html="highlightedDisplay"></div>
        </div>
      </div>
    </div>
    
    <!-- 历史记录侧边栏 -->
    <div v-if="showHistory" class="history-sidebar">
      <div class="history-header">
        <h3>历史记录</h3>
        <button @click="toggleHistory" class="close-btn">✕</button>
      </div>
      <div class="history-content">
        <div v-if="historyList.length === 0" class="history-empty">
          暂无历史记录
        </div>
        <div v-else>
          <div 
            v-for="(item, index) in historyList" 
            :key="index"
            class="history-item"
            @click="loadFromHistory(item)"
          >
            <div class="history-preview">
              {{ getHistoryPreview(item) }}
            </div>
            <div class="history-meta">
              <span class="history-time">{{ formatTime(item.timestamp) }}</span>
              <button @click.stop="removeFromHistory(index)" class="remove-btn">删除</button>
            </div>
          </div>
        </div>
      </div>
      <div class="history-footer">
        <button @click="clearHistory" :disabled="historyList.length === 0" class="clear-all-btn">
          清空历史
        </button>
      </div>
    </div>
    
    <!-- 历史记录遮罩 -->
    <div v-if="showHistory" class="history-overlay" @click="toggleHistory"></div>
    
    <!-- 右键菜单 -->
    <div 
      v-if="showContextMenu" 
      class="context-menu"
      :style="{ left: contextMenuPosition.x + 'px', top: contextMenuPosition.y + 'px' }"
      @click.stop
    >
      <div class="context-menu-item" @click="copyKey">
        <span class="menu-icon">📋</span>
        <span>复制 Key</span>
        <span class="menu-key">{{ contextMenuData?.key }}</span>
      </div>
      <div class="context-menu-item" @click="copyValue">
        <span class="menu-icon">📄</span>
        <span>复制 Value</span>
        <span class="menu-preview">{{ getValuePreview(contextMenuData?.value) }}</span>
      </div>
    </div>
    
    <!-- 右键菜单遮罩 -->
    <div v-if="showContextMenu" class="context-menu-overlay" @click="closeContextMenu"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onActivated, onUnmounted, defineComponent, h } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// 递归查找value
function getValueByPath(obj: any, path: (string|number)[]): any {
  return path.reduce((acc, key) => (acc !== undefined && acc !== null) ? acc[key] : undefined, obj)
}

// 获取值的预览文本
function getValuePreview(value: any): string {
  if (value === null) return 'null'
  if (value === undefined) return 'undefined'
  if (Array.isArray(value)) return `Array(${value.length})`
  if (typeof value === 'object') return `Object(${Object.keys(value).length})`
  if (typeof value === 'string') return value.length > 20 ? value.slice(0, 20) + '...' : value
  return String(value)
}

// JSON树节点组件
const JsonTreeNode = defineComponent({
  name: 'JsonTreeNode',
  props: {
    data: { type: [Object, Array, String, Number, Boolean], default: null },
    keyName: { type: String, default: '' },
    isRoot: { type: Boolean, default: false },
    level: { type: Number, default: 0 },
    selectedPath: { type: Array, default: () => [] },
    currentPath: { type: Array, default: () => [] }
  },
  emits: ['select', 'contextmenu'],
  setup(props, { emit }) {
    const isExpanded = ref(props.level < 2) // 默认展开前两层
    
    const toggleExpanded = () => {
      isExpanded.value = !isExpanded.value
    }
    
    const getValueType = (value: any): string => {
      if (value === null) return 'null'
      if (Array.isArray(value)) return 'array'
      return typeof value
    }
    
    const getValuePreview = (value: any): string => {
      if (value === null) return 'null'
      if (Array.isArray(value)) return `Array(${value.length})`
      if (typeof value === 'object') return `Object(${Object.keys(value).length})`
      if (typeof value === 'string') return `"${value}"`
      return String(value)
    }
    
    const getTypeIcon = (type: string): string => {
      switch (type) {
        case 'object': return '{ }'
        case 'array': return '[ ]'
        case 'string': return 'abc'
        case 'number': return '123'
        case 'boolean': return 'bool'
        case 'null': return 'null'
        default: return 'val'
      }
    }
    
    const getTypeBadge = (type: string, value: any): string => {
      if (type === 'array') return `Array(${value.length})`
      if (type === 'object' && value !== null) return `Object(${Object.keys(value).length})`
      if (type === 'string') return 'String'
      if (type === 'number') return 'Number'
      if (type === 'boolean') return 'Boolean'
      if (type === 'null') return 'Null'
      return type
    }
    
    const isExpandable = computed(() => props.data !== null && (Array.isArray(props.data) || typeof props.data === 'object'))
    const isSelected = computed(() => JSON.stringify(props.selectedPath) === JSON.stringify(props.currentPath))
    
    const handleSelect = (e: MouseEvent) => {
      e.stopPropagation()
      emit('select', props.currentPath.slice())
    }
    
    const handleRightClick = (e: MouseEvent) => {
      e.preventDefault()
      e.stopPropagation()
      
      // 发射右键菜单事件
      emit('contextmenu', {
        event: e,
        key: props.keyName,
        value: props.data,
        path: props.currentPath.slice()
      })
    }
    
    return () => {
      const { data, keyName, isRoot, level, selectedPath, currentPath } = props
      const valueType = getValueType(data)
      const indent = level * 16
      
      if (!isExpandable.value) {
        // 叶子节点
        return h('div', {
          class: ['tree-node', 'leaf-node', isSelected.value ? 'selected' : ''],
          style: { paddingLeft: `${indent}px` },
          onClick: handleSelect,
          onContextmenu: handleRightClick
        }, [
          // 添加类型图标
          h('span', { class: `type-icon ${valueType}` }, getTypeIcon(valueType)),
          h('span', { class: 'node-key' }, isRoot ? '' : keyName + ': '),
          h('span', { 
            class: `node-value ${valueType}` 
          }, getValuePreview(data)),
          // 添加类型标签
          h('span', { class: 'type-badge' }, getTypeBadge(valueType, data))
        ])
      }
      
      // 可展开节点
      const children: any[] = []
      
      // 节点头部
      children.push(
        h('div', {
          class: ['tree-node', 'expandable-node', isSelected.value ? 'selected' : ''],
          style: { paddingLeft: `${indent}px` },
          onClick: handleSelect,
          onContextmenu: handleRightClick
        }, [
          h('span', { 
            class: `expand-icon ${isExpanded.value ? 'expanded' : ''}`,
            onClick: (e: MouseEvent) => { e.stopPropagation(); toggleExpanded() }
          }, ''),
          h('span', { class: `type-icon ${valueType}` }, getTypeIcon(valueType)),
          !isRoot && h('span', { class: 'node-key' }, keyName + ': '),
          h('span', { 
            class: `node-value ${valueType}` 
          }, getValuePreview(data)),
          // 添加类型标签
          h('span', { class: 'type-badge' }, getTypeBadge(valueType, data))
        ])
      )
      
      // 子节点
      if (isExpanded.value) {
        const childNodes: any[] = []
        
        if (Array.isArray(data)) {
          data.forEach((item, index) => {
            childNodes.push(
              h(JsonTreeNode, {
                key: index,
                data: item,
                keyName: `[${index}]`,
                level: level + 1,
                selectedPath,
                currentPath: [...currentPath, index],
                onSelect: emit.bind(null, 'select'),
                onContextmenu: emit.bind(null, 'contextmenu')
              })
            )
          })
        } else if (typeof data === 'object' && data !== null) {
          Object.entries(data).forEach(([key, value]) => {
            childNodes.push(
              h(JsonTreeNode, {
                key: key,
                data: value,
                keyName: key,
                level: level + 1,
                selectedPath,
                currentPath: [...currentPath, key],
                onSelect: emit.bind(null, 'select'),
                onContextmenu: emit.bind(null, 'contextmenu')
              })
            )
          })
        }
        
        children.push(
          h('div', { class: 'tree-children' }, childNodes)
        )
      }
      
      return h('div', { class: 'tree-node-container' }, children)
    }
  }
})

const inputJson = ref('')
const parsedJson = ref<any>(null)
const error = ref('')

const clipboardStatus = ref<{type: 'success' | 'error' | 'info', message: string} | null>(null)
const selectedKeyPath = ref<(string|number)[]>([])
const showHistory = ref(false)

// 右键菜单相关
const showContextMenu = ref(false)
const contextMenuPosition = ref({ x: 0, y: 0 })
const contextMenuData = ref<{ key: string, value: any, path: (string|number)[] } | null>(null)

// 历史记录类型定义
interface HistoryItem {
  data: any
  timestamp: number
  hash: string
}

const historyList = ref<HistoryItem[]>([])

const selectedValue = computed(() => {
  if (!parsedJson.value || !selectedKeyPath.value.length) return undefined
  return getValueByPath(parsedJson.value, selectedKeyPath.value)
})

// 显示的值：如果有选中的key则显示选中的值，否则显示完整JSON
const displayValue = computed(() => {
  if (selectedValue.value !== undefined) {
    return typeof selectedValue.value === 'object' 
      ? JSON.stringify(selectedValue.value, null, 2)
      : String(selectedValue.value)
  }
  return parsedJson.value ? JSON.stringify(parsedJson.value, null, 2) : ''
})

// 高亮显示的内容
const highlightedDisplay = computed(() => {
  if (selectedValue.value !== undefined) {
    if (typeof selectedValue.value === 'object') {
      return syntaxHighlight(JSON.stringify(selectedValue.value, null, 2))
    }
    return syntaxHighlight(JSON.stringify(selectedValue.value))
  }
  return parsedJson.value ? syntaxHighlight(JSON.stringify(parsedJson.value, null, 2)) : ''
})

function onSelectKey(path: (string|number)[]) {
  selectedKeyPath.value = path
}

// 处理右键菜单
function onContextMenu(event: { event: MouseEvent, key: string, value: any, path: (string|number)[] }) {
  showContextMenu.value = true
  contextMenuPosition.value = { x: event.event.clientX, y: event.event.clientY }
  contextMenuData.value = { 
    key: event.key, 
    value: event.value, 
    path: event.path 
  }
}

// 关闭右键菜单
function closeContextMenu() {
  showContextMenu.value = false
  contextMenuData.value = null
}

// 复制key
async function copyKey() {
  if (!contextMenuData.value) return
  try {
    await navigator.clipboard.writeText(contextMenuData.value.key)
    showClipboardStatus('success', '键名已复制到剪贴板！')
  } catch (err) {
    showClipboardStatus('error', '复制失败')
  }
  closeContextMenu()
}

// 复制value
async function copyValue() {
  if (!contextMenuData.value) return
  try {
    const valueStr = typeof contextMenuData.value.value === 'string' 
      ? contextMenuData.value.value 
      : JSON.stringify(contextMenuData.value.value, null, 2)
    await navigator.clipboard.writeText(valueStr)
    showClipboardStatus('success', '值已复制到剪贴板！')
  } catch (err) {
    showClipboardStatus('error', '复制失败')
  }
  closeContextMenu()
}

async function copySelectedValue() {
  if (!parsedJson.value) return
  let val = displayValue.value
  try {
    await navigator.clipboard.writeText(val)
    showClipboardStatus('success', '已复制到剪贴板！')
  } catch (err) {
    showClipboardStatus('error', '复制失败')
  }
}

// 历史记录相关函数
function generateHash(data: any): string {
  // 使用简单的哈希算法代替btoa，避免中文字符编码问题
  const str = JSON.stringify(data)
  let hash = 0
  for (let i = 0; i < str.length; i++) {
    const char = str.charCodeAt(i)
    hash = ((hash << 5) - hash) + char
    hash = hash & hash // 转换为32位整数
  }
  return Math.abs(hash).toString(36).slice(0, 16)
}

function addToHistory(data: any) {
  const hash = generateHash(data)
  
  // 检查是否已存在相同的JSON
  const existingIndex = historyList.value.findIndex(item => item.hash === hash)
  
  if (existingIndex !== -1) {
    // 如果已存在，不做任何操作，保持原来的时间戳和位置
    return
  }
  
  // 如果不存在，创建新记录
  const historyItem: HistoryItem = {
    data,
    timestamp: Date.now(),
    hash
  }
  
  // 添加新记录
  historyList.value.push(historyItem)
  
  // 按时间戳排序（最新在前）
  historyList.value.sort((a, b) => b.timestamp - a.timestamp)
  
  // 最多保存20条
  if (historyList.value.length > 20) {
    historyList.value = historyList.value.slice(0, 20)
  }
}

function toggleHistory() {
  showHistory.value = !showHistory.value
}

function loadFromHistory(item: HistoryItem) {
  parsedJson.value = item.data
  inputJson.value = JSON.stringify(item.data, null, 2)
  selectedKeyPath.value = []
  error.value = ''
  showHistory.value = false
  showClipboardStatus('success', '已加载历史记录')
}

function removeFromHistory(index: number) {
  historyList.value.splice(index, 1)
}

function clearHistory() {
  historyList.value = []
  showClipboardStatus('info', '历史记录已清空')
}

function getHistoryPreview(item: HistoryItem): string {
  const str = JSON.stringify(item.data)
  return str.length > 100 ? str.slice(0, 100) + '...' : str
}

function formatTime(timestamp: number): string {
  const date = new Date(timestamp)
  const now = new Date()
  const isCurrentYear = date.getFullYear() === now.getFullYear()
  
  if (isCurrentYear) {
    // 当年的记录只显示月日时分秒
    return date.toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
  } else {
    // 不同年份的记录显示完整日期时分秒
    return date.toLocaleString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    })
  }
}



// 检查字符串是否为有效JSON
const isValidJson = (str: string): boolean => {
  try {
    JSON.parse(str)
    return true
  } catch {
    return false
  }
}

// 处理全局快捷键事件
const setupGlobalShortcutListeners = async () => {
  try {
    // 监听全局快捷键触发的JSON格式剪贴板事件
    await listen('global-clipboard-json', (event) => {
      const clipboardText = event.payload as string
      try {
        parsedJson.value = JSON.parse(clipboardText)
        inputJson.value = clipboardText
        error.value = ''
        addToHistory(parsedJson.value)
        showClipboardStatus('success', '🌍 全局快捷键检测到JSON格式，已自动解析')
      } catch (err) {
        error.value = err instanceof Error ? err.message : '无效的JSON格式'
        parsedJson.value = null
        showClipboardStatus('error', '🌍 全局快捷键: JSON解析失败')
      }
    })
    
    // 监听全局快捷键触发的非JSON格式剪贴板事件
    await listen('global-clipboard-not-json', (_event) => {
      showClipboardStatus('error', '🌍 全局快捷键: 剪贴板内容不是有效的JSON格式')
    })
    
    console.log('Global shortcut event listeners registered')
  } catch (err) {
    console.error('Failed to setup global shortcut listeners:', err)
  }
}

// 获取剪贴板内容并自动解析
const getClipboardContent = async () => {
  try {
    console.log('开始获取剪贴板内容...')
    const clipboardText = await invoke<string>('get_clipboard')
    console.log('剪贴板内容:', clipboardText)
    
    if (!clipboardText || !clipboardText.trim()) {
      showClipboardStatus('info', '剪贴板为空')
      return
    }
    
    // 检查是否为有效JSON并直接解析
    if (isValidJson(clipboardText)) {
      try {
        const parsed = JSON.parse(clipboardText)
        parsedJson.value = parsed
        inputJson.value = clipboardText
        error.value = ''
        addToHistory(parsed)
        showClipboardStatus('success', '检测到JSON格式，已自动解析')
      } catch (err) {
        error.value = err instanceof Error ? err.message : '无效的JSON格式'
        parsedJson.value = null
        showClipboardStatus('error', '解析JSON失败')
      }
    } else {
      showClipboardStatus('error', '剪贴板内容不是有效的JSON格式')
    }
  } catch (err) {
    console.error('获取剪贴板失败详细信息:', err)
    const errorMessage = err instanceof Error ? err.message : String(err)
    showClipboardStatus('error', '获取剪贴板失败: ' + errorMessage)
  }
}

// 显示剪贴板状态
const showClipboardStatus = (type: 'success' | 'error' | 'info', message: string) => {
  clipboardStatus.value = { type, message }
  setTimeout(() => {
    clipboardStatus.value = null
  }, 4000)
}

// 自动检查剪贴板（页面加载时）
const autoCheckClipboard = async () => {
  try {
    const clipboardText = await invoke<string>('get_clipboard')
    
    if (clipboardText && clipboardText.trim() && isValidJson(clipboardText)) {
      try {
        parsedJson.value = JSON.parse(clipboardText)
        inputJson.value = clipboardText
        error.value = ''
        addToHistory(parsedJson.value)
        showClipboardStatus('success', '自动检测到剪贴板中的JSON格式数据')
      } catch (err) {
        // 静默处理错误
        console.log('自动解析JSON失败:', err)
      }
    }
  } catch (err) {
    // 静默处理错误，不影响正常使用
    console.log('自动检查剪贴板失败:', err)
  }
}

// JSON语法高亮
const syntaxHighlight = (json: string): string => {
  json = json.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
  
  return json.replace(/("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g, (match) => {
    let cls = 'json-number'
    if (/^"/.test(match)) {
      if (/:$/.test(match)) {
        cls = 'json-key'
      } else {
        cls = 'json-string'
      }
    } else if (/true|false/.test(match)) {
      cls = 'json-boolean'
    } else if (/null/.test(match)) {
      cls = 'json-null'
    }
    return '<span class="' + cls + '">' + match + '</span>'
  })
}

// 键盘事件处理器（本地快捷键作为备用）
const handleKeydown = (event: KeyboardEvent) => {
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0
  const isModifierPressed = isMac ? event.metaKey : event.ctrlKey
  
  // Command+Shift+G (Mac) 或 Ctrl+Shift+G (Windows/Linux) - 全局快捷键备用
  if (isModifierPressed && event.shiftKey && event.key.toLowerCase() === 'g') {
    event.preventDefault()
    event.stopPropagation()
    getClipboardContent()
    return
  }
  
  // Command+V (Mac) 或 Ctrl+V (Windows/Linux) - 传统粘贴快捷键
  if (isModifierPressed && event.key.toLowerCase() === 'v') {
    // 检查当前焦点是否在输入框或文本区域
    const activeElement = document.activeElement as HTMLElement
    const isInputFocused = activeElement && (
      activeElement.tagName === 'INPUT' || 
      activeElement.tagName === 'TEXTAREA' || 
      (activeElement as any).contentEditable === 'true'
    )
    
    // 如果没有焦点在输入框上，则拦截粘贴事件并处理JSON
    if (!isInputFocused) {
      event.preventDefault()
      event.stopPropagation()
      getClipboardContent()
    }
  }
}

// 生命周期钩子
onMounted(() => {
  // 页面加载时自动检查剪贴板
  autoCheckClipboard()
  // 添加键盘事件监听器（本地快捷键作为备用）
  document.addEventListener('keydown', handleKeydown)
  // 设置全局快捷键事件监听器
  setupGlobalShortcutListeners()
})

// 组件激活时（用于keep-alive情况）
onActivated(() => {
  // 组件激活时也检查剪贴板
  autoCheckClipboard()
  // 确保键盘事件监听器存在
  document.addEventListener('keydown', handleKeydown)
  // 重新设置全局快捷键事件监听器
  setupGlobalShortcutListeners()
})

// 组件卸载时移除事件监听器
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.json-parser {
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
  height: calc(100vh - 80px);
  display: flex;
  flex-direction: column;
  background: #1e1e1e;
  color: #d4d4d4;
}

.header {
  text-align: center;
  margin-bottom: 20px;
}

.header h1 {
  color: #ffffff;
  margin-bottom: 10px;
}

.header p {
  color: #a0a0a0;
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
}

.section-header h3 {
  margin: 0;
  font-size: 16px;
}

.actions {
  display: flex;
  gap: 10px;
}

.copy-btn,
.compact-btn,
.clipboard-btn,
.history-btn {
  background: rgba(255, 255, 255, 0.2);
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.3);
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s ease;
  position: relative;
}

.copy-btn:hover:not(:disabled),
.compact-btn:hover:not(:disabled),
.clipboard-btn:hover,
.history-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.clipboard-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  position: relative;
}

.shortcut-hint {
  font-size: 11px;
  opacity: 0.8;
  background: rgba(255, 255, 255, 0.1);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: monospace;
}

.global-indicator {
  font-size: 12px;
  position: absolute;
  top: -2px;
  right: -2px;
  background: #28a745;
  color: white;
  border-radius: 50%;
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { transform: scale(1); }
  50% { transform: scale(1.1); }
  100% { transform: scale(1); }
}

.json-tree-container {
  flex: 1;
  padding: 8px 12px;
  background: #1e1e1e;
  overflow: auto;
  color: #cccccc;
}

.json-tree {
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', 'Source Code Pro', monospace;
  font-size: 13px;
  line-height: 1.4;
  color: #cccccc;
}

:deep(.tree-node) {
  display: flex;
  align-items: center;
  padding: 1px 4px;
  cursor: pointer;
  user-select: none;
  color: #cccccc;
  min-height: 22px;
  border-radius: 3px;
  margin: 1px 0;
  transition: background-color 0.1s ease;
  position: relative;
  width: 100%;
}

:deep(.tree-node.leaf-node) {
  cursor: pointer;
}

:deep(.tree-node.selected) {
  background: #094771;
  color: #ffffff;
}

:deep(.tree-node.selected .type-badge) {
  background: rgba(255, 255, 255, 0.2);
  color: #ffffff;
  border-color: rgba(255, 255, 255, 0.2);
}

:deep(.tree-node:hover:not(.selected)) {
  background: #2a2d2e;
}

:deep(.tree-node:hover:not(.selected) .type-badge) {
  background: rgba(255, 255, 255, 0.15);
  border-color: rgba(255, 255, 255, 0.15);
}

:deep(.expand-icon) {
  width: 16px;
  height: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-right: 2px;
  color: #cccccc;
  font-size: 10px;
  transition: transform 0.1s ease;
  cursor: pointer;
  position: relative;
}

:deep(.expand-icon::before) {
  content: '▶';
  transition: transform 0.1s ease;
}

:deep(.expand-icon.expanded::before) {
  transform: rotate(90deg);
}

:deep(.type-icon) {
  min-width: 28px;
  height: 16px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  margin-right: 4px;
  font-size: 9px;
  font-weight: 600;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  border-radius: 2px;
  padding: 1px 3px;
}

:deep(.type-icon.object) {
  background: rgba(156, 220, 254, 0.15);
  color: #9cdcfe;
  border: 1px solid rgba(156, 220, 254, 0.3);
}

:deep(.type-icon.array) {
  background: rgba(156, 220, 254, 0.15);
  color: #9cdcfe;
  border: 1px solid rgba(156, 220, 254, 0.3);
}

:deep(.type-icon.string) {
  background: rgba(206, 145, 120, 0.15);
  color: #ce9178;
  border: 1px solid rgba(206, 145, 120, 0.3);
}

:deep(.type-icon.number) {
  background: rgba(181, 206, 168, 0.15);
  color: #b5cea8;
  border: 1px solid rgba(181, 206, 168, 0.3);
}

:deep(.type-icon.boolean) {
  background: rgba(86, 156, 214, 0.15);
  color: #569cd6;
  border: 1px solid rgba(86, 156, 214, 0.3);
}

:deep(.type-icon.null) {
  background: rgba(86, 156, 214, 0.15);
  color: #569cd6;
  border: 1px solid rgba(86, 156, 214, 0.3);
}

:deep(.type-badge) {
  margin-left: auto;
  padding: 1px 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  font-size: 10px;
  color: #a0a0a0;
  font-weight: 500;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

:deep(.node-key) {
  color: #9cdcfe;
  font-weight: 400;
  margin-right: 4px;
}

:deep(.node-value) {
  margin-left: 2px;
  font-weight: 400;
}

:deep(.node-value.string) {
  color: #ce9178;
}

:deep(.node-value.number) {
  color: #b5cea8;
}

:deep(.node-value.boolean) {
  color: #569cd6;
  font-weight: 400;
}

:deep(.node-value.null) {
  color: #569cd6;
  font-weight: 400;
}

:deep(.node-value.object),
:deep(.node-value.array) {
  color: #cccccc;
  font-style: normal;
  opacity: 0.8;
}

:deep(.tree-children) {
  margin-left: 12px;
}

:deep(.tree-node-container) {
  position: relative;
}

.json-output {
  flex: 1;
  padding: 20px;
  background: #1e1e1e;
  overflow: auto;
  color: #d4d4d4;
}

.json-display {
  font-family: 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
}

.placeholder {
  color: #6a6a6a;
  text-align: center;
  padding: 40px 20px;
  font-style: italic;
}

.placeholder-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.placeholder p {
  margin: 8px 0;
}

.placeholder-hint {
  font-size: 12px;
  opacity: 0.7;
}

.placeholder kbd {
  background: #3c3c3c;
  border: 1px solid #555;
  border-radius: 3px;
  padding: 2px 6px;
  font-size: 12px;
  color: #ffffff;
  font-family: monospace;
}

.error-message {
  background: #3c1e1e;
  color: #f48771;
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

.clipboard-status {
  padding: 10px 20px;
  font-size: 14px;
  border-top: 1px solid #3c3c3c;
  animation: fadeInOut 4s ease-in-out;
}

.clipboard-status.success {
  background: #1e3a1e;
  color: #4caf50;
  border-left: 4px solid #28a745;
}

.clipboard-status.error {
  background: #3c1e1e;
  color: #f48771;
  border-left: 4px solid #dc3545;
}

.clipboard-status.info {
  background: #1e2a3c;
  color: #64b5f6;
  border-left: 4px solid #17a2b8;
}

@keyframes fadeInOut {
  0% { opacity: 0; transform: translateY(-10px); }
  10% { opacity: 1; transform: translateY(0); }
  90% { opacity: 1; transform: translateY(0); }
  100% { opacity: 0; transform: translateY(-10px); }
}

.info-panel {
  margin-top: 20px;
  padding: 15px;
  background: #252526;
  border-radius: 8px;
  border-left: 4px solid #42b983;
  border: 1px solid #3c3c3c;
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 20px;
}

.stats h4,
.keyboard-shortcuts h4 {
  margin: 0 0 15px 0;
  color: #ffffff;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  padding: 5px 0;
  border-bottom: 1px solid #3c3c3c;
}

.stat-item:last-child {
  border-bottom: none;
}

.stat-item span:first-child {
  font-weight: 500;
  color: #a0a0a0;
}

.stat-item span:last-child {
  color: #ffffff;
  font-weight: 600;
}

.keyboard-shortcuts {
  min-width: 220px;
}

.shortcut-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  color: #a0a0a0;
  font-size: 14px;
}

.shortcut-item kbd {
  background: #3c3c3c;
  border: 1px solid #555;
  border-radius: 3px;
  padding: 2px 6px;
  font-size: 12px;
  color: #ffffff;
  box-shadow: 0 1px 2px rgba(0,0,0,0.3);
  font-family: monospace;
}

.shortcut-item span {
  color: #ffffff;
  font-weight: 500;
}

.shortcut-note {
  font-size: 12px;
  color: #28a745;
  margin-top: 8px;
  padding: 4px 8px;
  background: rgba(40, 167, 69, 0.1);
  border-radius: 4px;
  border-left: 3px solid #28a745;
}

/* JSON语法高亮样式 - 暗黑主题 */
:deep(.json-key) {
  color: #9cdcfe;
  font-weight: bold;
}

:deep(.json-string) {
  color: #ce9178;
}

:deep(.json-number) {
  color: #b5cea8;
}

:deep(.json-boolean) {
  color: #569cd6;
  font-weight: bold;
}

:deep(.json-null) {
  color: #569cd6;
  font-weight: bold;
}

/* 历史记录样式 */
.history-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 1000;
}

.history-sidebar {
  position: fixed;
  top: 0;
  right: 0;
  width: 400px;
  height: 100vh;
  background: #252526;
  border-left: 1px solid #3c3c3c;
  z-index: 1001;
  display: flex;
  flex-direction: column;
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #3c3c3c;
  background: #2d2d30;
}

.history-header h3 {
  margin: 0;
  color: #ffffff;
}

.close-btn {
  background: none;
  border: none;
  color: #cccccc;
  font-size: 18px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: background 0.2s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.history-content {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.history-empty {
  text-align: center;
  color: #6a6a6a;
  padding: 40px 20px;
  font-style: italic;
}

.history-item {
  background: #1e1e1e;
  border: 1px solid #3c3c3c;
  border-radius: 6px;
  margin-bottom: 10px;
  padding: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.history-item:hover {
  background: #2a2a2a;
  border-color: #4a4a4a;
}

.history-preview {
  color: #d4d4d4;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.4;
  margin-bottom: 8px;
  word-break: break-all;
}

.history-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.history-time {
  color: #a0a0a0;
  font-size: 11px;
}

.remove-btn {
  background: #dc3545;
  color: white;
  border: none;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 11px;
  cursor: pointer;
  transition: background 0.2s;
}

.remove-btn:hover {
  background: #c82333;
}

.history-footer {
  padding: 20px;
  border-top: 1px solid #3c3c3c;
  background: #2d2d30;
}

.clear-all-btn {
  width: 100%;
  background: #6c757d;
  color: white;
  border: none;
  padding: 10px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
}

.clear-all-btn:hover:not(:disabled) {
  background: #5a6268;
}

.clear-all-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .content {
    grid-template-columns: 1fr;
    grid-template-rows: 1fr 1fr;
  }
  
  .json-parser {
    padding: 10px;
  }
  
  .actions {
    flex-direction: column;
    gap: 5px;
  }
  
  .section-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 10px;
  }
  
  .shortcut-hint {
    display: none;
  }
  
  .global-indicator {
    display: none;
  }
  
  .history-sidebar {
    width: 100%;
  }
}

/* 右键菜单样式 */
.context-menu {
  position: fixed;
  background: #252526;
  border: 1px solid #3c3c3c;
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
  z-index: 2000;
  min-width: 180px;
  overflow: hidden;
  animation: contextMenuFadeIn 0.15s ease-out;
}

@keyframes contextMenuFadeIn {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-5px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

.context-menu-item {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  cursor: pointer;
  color: #d4d4d4;
  font-size: 13px;
  transition: background-color 0.2s ease;
  gap: 8px;
}

.context-menu-item:hover {
  background: #094771;
  color: #ffffff;
}

.context-menu-item:not(:last-child) {
  border-bottom: 1px solid #3c3c3c;
}

.menu-icon {
  font-size: 14px;
  width: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.menu-key {
  margin-left: auto;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 11px;
  color: #9cdcfe;
  background: rgba(156, 220, 254, 0.1);
  padding: 2px 6px;
  border-radius: 3px;
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.menu-preview {
  margin-left: auto;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 11px;
  color: #a0a0a0;
  background: rgba(160, 160, 160, 0.1);
  padding: 2px 6px;
  border-radius: 3px;
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.context-menu-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 1999;
  background: transparent;
}
</style> 