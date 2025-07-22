<template>
  <div class="json-parser">
    <div class="header">
      <h1>JSON 解析与格式化工具</h1>
      <p>使用快捷键或按钮从剪贴板加载JSON数据（支持带注释和多余逗号的JSON）</p>
    </div>
    <div class="content">
      <div class="input-section">
        <div class="section-header">
          <h3>JSON 结构视图</h3>
          <div class="actions">
            <button @click="getClipboardContent" class="clipboard-btn" :title="'获取剪贴板 (全局快捷键: ⌘⇧G)'"><span>获取剪贴板</span><span class="shortcut-hint">⌘⇧G</span><span class="global-indicator">🌍</span></button>
            <input
              v-model="searchText"
              placeholder="搜索 key 或 value..."
              style="background: #23272f; color: #e0e0e0; border: 1.5px solid #333a45; border-radius: 6px; padding: 6px 12px; font-size: 15px; min-width: 180px; outline: none; margin-left: 12px;"
            />
          </div>
        </div>
        <div class="json-tree-container">
          <div v-if="error" class="error-message">
            <h4>解析错误：</h4>
            <p>{{ error }}</p>
          </div>
          <div v-else-if="filteredJson" class="json-tree">
            <JsonTreeNode 
              :data="filteredJson" 
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
            <p class="placeholder-hint">💡 支持带注释和多余逗号的JSON格式</p>
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
            <button @click="copySelectedValue" :disabled="!parsedJson" class="copy-btn" title="复制选中节点的值 (⌘C / Ctrl+C)">
              复制值
              <span class="shortcut-hint">⌘C</span>
            </button>
            <button @click="escapeClipboardJson" class="copy-btn" title="从剪贴板获取数据并转义引号">
              转义剪贴板
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
      <div class="context-menu-item" @click="editValue">
        <span class="menu-icon">✏️</span>
        <span>编辑 Value</span>
        <span class="menu-preview">{{ getValuePreview(contextMenuData?.value) }}</span>
      </div>
    </div>
    
    <!-- 右键菜单遮罩 -->
    <div v-if="showContextMenu" class="context-menu-overlay" @click="closeContextMenu"></div>
    
    <!-- 编辑值弹窗 -->
    <div v-if="showEditDialog" class="edit-dialog-overlay" @click="closeEditDialog">
      <div class="edit-dialog" @click.stop>
        <div class="edit-dialog-header">
          <h3>编辑值</h3>
        </div>
        <div class="edit-dialog-content">
          <div class="edit-field">
            <label>键名:</label>
            <input 
              v-model="editForm.key" 
              type="text" 
              readonly 
              class="readonly-input"
            />
          </div>
          <div class="edit-field">
            <label>新值:</label>
            <textarea 
              v-model="editForm.value" 
              placeholder="请输入新的值..."
              rows="6"
              class="edit-textarea"
            ></textarea>
          </div>
        </div>
        <div class="edit-dialog-footer">
          <button @click="closeEditDialog" class="cancel-btn">取消</button>
          <button @click="saveEditValue" class="save-btn">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onActivated, onUnmounted, defineComponent, h, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import '../assets/styles/JsonParser.css'

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

async function escapeClipboardJson() {
  try {
    // 从剪贴板获取文本
    const clipboardText = await invoke<string>('get_clipboard')
    
    if (!clipboardText || !clipboardText.trim()) {
      showClipboardStatus('info', '剪贴板为空')
      return
    }
    
    // 使用JSON.stringify来正确处理所有转义字符
    // 这样可以确保换行符、引号、反斜杠等都被正确转义
    const escapedText = JSON.stringify(clipboardText)
    
    // 将转义后的文本复制回剪贴板
    await navigator.clipboard.writeText(escapedText)
    
    // 尝试解析转义后的JSON以验证格式
    try {
      const testJson = JSON.parse(escapedText)
      showClipboardStatus('success', '已转义所有特殊字符并复制到剪贴板！')
    } catch (parseErr) {
      showClipboardStatus('info', '已转义特殊字符，但JSON格式可能仍有问题')
    }
    
  } catch (err) {
    console.error('转义剪贴板失败:', err)
    showClipboardStatus('error', '转义剪贴板失败: ' + (err instanceof Error ? err.message : String(err)))
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

// 保存历史记录到文件
async function saveHistoryToFile() {
  try {
    await invoke('save_json_history', { history: historyList.value })
    console.log('历史记录已保存到文件')
  } catch (err) {
    console.error('保存历史记录失败:', err)
    showClipboardStatus('error', '保存历史记录失败')
  }
}

// 从文件加载历史记录
async function loadHistoryFromFile() {
  try {
    const history = await invoke<HistoryItem[]>('load_json_history')
    historyList.value = history
    console.log('已从文件加载历史记录')
  } catch (err) {
    console.error('加载历史记录失败:', err)
    showClipboardStatus('error', '加载历史记录失败')
  }
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

  // 保存到文件
  saveHistoryToFile()
}

function removeFromHistory(index: number) {
  historyList.value.splice(index, 1)
  // 保存到文件
  saveHistoryToFile()
}

function clearHistory() {
  historyList.value = []
  showClipboardStatus('info', '历史记录已清空')
  // 保存到文件
  saveHistoryToFile()
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

// 检查是否有从PostParser跳转过来的临时JSON数据
function checkTempJsonData() {
  try {
    console.log('JsonParser: 检查临时JSON数据...')
    const tempData = localStorage.getItem('tempJsonData')
    console.log('JsonParser: 从localStorage获取的临时数据:', tempData ? '有数据' : '无数据')
    
    if (tempData) {
      console.log('JsonParser: 开始处理临时数据')
      // 清除localStorage中的临时数据
      localStorage.removeItem('tempJsonData')
      
      // 解析并加载JSON数据
      const jsonData = parseJsonWithComments(tempData)
      parsedJson.value = jsonData
      inputJson.value = tempData
      selectedKeyPath.value = []
      error.value = ''
      
      // 添加到历史记录（这里会自动调用saveHistoryToFile）
      addToHistory(jsonData)
      
      // 显示成功状态
      showClipboardStatus('success', '✨ 已从HTTP请求解析工具加载JSON数据')
      console.log('JsonParser: 临时数据处理完成')
    } else {
      console.log('JsonParser: 没有临时数据需要处理')
    }
  } catch (err) {
    console.error('JsonParser: 加载临时JSON数据失败:', err)
    showClipboardStatus('error', '加载临时JSON数据失败')
    // 即使失败也要清除localStorage
    localStorage.removeItem('tempJsonData')
  }
}

/**
 * 清理JSON字符串中的注释和多余逗号
 * @param jsonStr 原始JSON字符串
 * @returns 清理后的JSON字符串
 */
const cleanJsonString = (jsonStr: string): string => {
  return jsonStr
    // 移除单行注释 // 注释内容
    .replace(/\/\/.*$/gm, '')
    // 移除多行注释 /* 注释内容 */
    .replace(/\/\*[\s\S]*?\*\//g, '')
    // 清理多余的空行
    .replace(/\n\s*\n/g, '\n')
    // 移除连续逗号
    .replace(/,(\s*,)/g, ',')
    // 移除尾随逗号（对象和数组结尾）
    .replace(/,(\s*[}\]])/g, '$1')
    // 移除行尾孤立逗号
    .replace(/,(\s*\n\s*[}\]])/g, '$1')
    // 移除属性值后的多余逗号
    .replace(/,(\s*\n\s*})/g, '$1')
    .replace(/,(\s*\n\s*\])/g, '$1')
    // 最终清理：移除所有在 } 或 ] 之前的逗号
    .replace(/,(\s*[}\]])/g, '$1')
    .trim()
}

// 解析带注释的JSON
const parseJsonWithComments = (jsonStr: string): any => {
  const cleanedJson = cleanJsonString(jsonStr)
  
  // 调试：如果原始JSON和清理后的JSON不同，在控制台输出
  if (jsonStr !== cleanedJson) {
    console.log('JSON清理前后对比:')
    console.log('原始JSON:', jsonStr)
    console.log('清理后JSON:', cleanedJson)
  }
  
  return JSON.parse(cleanedJson)
}

// 检查字符串是否为有效JSON（支持注释）
const isValidJson = (str: string): boolean => {
  try {
    parseJsonWithComments(str)
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
        const parsed = parseJsonWithComments(clipboardText)
        parsedJson.value = parsed
        inputJson.value = clipboardText
        error.value = ''
        // 添加到历史记录（这里会自动调用saveHistoryToFile）
        addToHistory(parsed)
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
        const parsed = parseJsonWithComments(clipboardText)
        parsedJson.value = parsed
        inputJson.value = clipboardText
        error.value = ''
        // 添加到历史记录（这里会自动调用saveHistoryToFile）
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
  // 如果当前已有解析的JSON，则不执行自动检查，避免覆盖用户正在查看的内容
  if (parsedJson.value) {
    console.log('JsonParser: 已有JSON数据，跳过自动剪贴板检查。');
    return;
  }

  try {
    const clipboardText = await invoke<string>('get_clipboard')
    
    if (clipboardText && clipboardText.trim() && isValidJson(clipboardText)) {
      try {
        parsedJson.value = parseJsonWithComments(clipboardText)
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
  // 这个快捷键应该总是可用，无论焦点在哪里
  if (isModifierPressed && event.shiftKey && event.key.toLowerCase() === 'g') {
    event.preventDefault()
    event.stopPropagation()
    getClipboardContent()
    return
  }

  // 检查当前焦点是否在输入框或文本区域
  const activeElement = document.activeElement as HTMLElement
  const isInputFocused = activeElement && (
    activeElement.tagName === 'INPUT' ||
    activeElement.tagName === 'TEXTAREA' ||
    (activeElement as any).contentEditable === 'true'
  )

  // 如果焦点在输入框中，则不执行下面的自定义快捷键
  if (isInputFocused) {
    return
  }
  
  // Command+V (Mac) 或 Ctrl+V (Windows/Linux) - 传统粘贴快捷键
  if (isModifierPressed && !event.shiftKey && event.key.toLowerCase() === 'v') {
    // 如果没有焦点在输入框上，则拦截粘贴事件并处理JSON
    event.preventDefault()
    event.stopPropagation()
    getClipboardContent()
  }

  // Command+C (Mac) 或 Ctrl+C (Windows/Linux) - 复制选中值
  if (isModifierPressed && !event.shiftKey && event.key.toLowerCase() === 'c') {
    // 如果没有焦点在输入框上，则拦截复制事件并执行自定义复制
    event.preventDefault()
    event.stopPropagation()
    copySelectedValue()
  }
}

// 生命周期钩子
onMounted(async () => {
  // 从文件加载历史记录
  await loadHistoryFromFile()
  // 恢复上一次的查看状态
  loadCurrentState()
  // 检查是否有从PostParser跳转过来的临时JSON数据
  checkTempJsonData()
  // 页面加载时自动检查剪贴板
  autoCheckClipboard()
  // 添加键盘事件监听器（本地快捷键作为备用）
  document.addEventListener('keydown', handleKeydown)
  // 设置全局快捷键事件监听器
  setupGlobalShortcutListeners()
  // 监听 Option+Shift+F/Alt+Shift+F 后端处理事件
  listen('process-clipboard-done', (event) => {
    const msg = event.payload as string
    showClipboardStatus('info', msg)
  })
})

// 组件激活时（用于keep-alive情况）
onActivated(async () => {
  // 从文件加载最新的历史记录
  await loadHistoryFromFile();
  // 恢复上一次的查看状态
  loadCurrentState();
  // 检查是否有从PostParser跳转过来的临时JSON数据
  checkTempJsonData();
  // 组件激活时也检查剪贴板
  autoCheckClipboard();
  // 确保键盘事件监听器存在
  document.addEventListener('keydown', handleKeydown);
  // 重新设置全局快捷键事件监听器
  setupGlobalShortcutListeners();
})

// 组件卸载时移除事件监听器
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

// 监听数据变化，自动保存状态
watch([parsedJson, inputJson, selectedKeyPath, historyList], () => {
  saveCurrentState()
}, { deep: true })

// 保存当前状态到localStorage
function saveCurrentState() {
  if (parsedJson.value) {
    const currentState = {
      parsedJson: parsedJson.value,
      inputJson: inputJson.value,
      selectedKeyPath: selectedKeyPath.value,
      timestamp: Date.now()
    }
    localStorage.setItem('jsonParserState', JSON.stringify(currentState))
    console.log('JsonParser: 状态已保存')
  }
}

// 恢复当前状态从localStorage
function loadCurrentState() {
  try {
    const savedStateJSON = localStorage.getItem('jsonParserState');
    if (savedStateJSON) {
      const savedState = JSON.parse(savedStateJSON);
      
      // 只有当有有效数据时才恢复
      if (savedState.parsedJson) {
        parsedJson.value = savedState.parsedJson;
        inputJson.value = savedState.inputJson;
        selectedKeyPath.value = savedState.selectedKeyPath || [];
        console.log('JsonParser: 状态已从localStorage恢复');
      }
    }
  } catch (err) {
    console.error('JsonParser: 从localStorage恢复状态失败:', err);
    // 清除可能损坏的状态
    localStorage.removeItem('jsonParserState');
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
  saveCurrentState()
}

// 编辑值弹窗相关
const showEditDialog = ref(false)
const editForm = ref({
  key: '',
  value: '',
  type: 'string'
})

function closeEditDialog() {
  showEditDialog.value = false
}

function saveEditValue() {
  if (!contextMenuData.value) return
  try {
    let newValue: any
    
    // 根据选择的类型解析值
    switch (editForm.value.type) {
      case 'string':
        newValue = editForm.value.value
        break
      case 'number':
        newValue = Number(editForm.value.value)
        if (isNaN(newValue)) {
          throw new Error('无效的数字格式')
        }
        break
      case 'boolean':
        if (editForm.value.value.toLowerCase() === 'true') {
          newValue = true
        } else if (editForm.value.value.toLowerCase() === 'false') {
          newValue = false
        } else {
          throw new Error('无效的布尔值格式')
        }
        break
      case 'null':
        if (editForm.value.value.toLowerCase() === 'null') {
          newValue = null
        } else if (editForm.value.value.toLowerCase() === 'true') {
          newValue = true
        } else if (editForm.value.value.toLowerCase() === 'false') {
          newValue = false
        } else if (!isNaN(Number(editForm.value.value))) {
          newValue = Number(editForm.value.value)
        } else {
          newValue = editForm.value.value
        }
        break
      case 'object':
      case 'array':
        newValue = JSON.parse(editForm.value.value)
        break
      default:
        newValue = editForm.value.value
    }
    
    // 更新JSON数据中的值
    updateJsonValue(contextMenuData.value.path, newValue)
    
    showClipboardStatus('success', '值已更新！')
  } catch (err) {
    showClipboardStatus('error', '更新值失败: ' + (err instanceof Error ? err.message : String(err)))
  }
  closeEditDialog()
}

function editValue() {
  if (!contextMenuData.value) return
  
  // 设置编辑表单的初始值
  editForm.value.key = contextMenuData.value.key
  editForm.value.value = typeof contextMenuData.value.value === 'string' 
    ? contextMenuData.value.value 
    : JSON.stringify(contextMenuData.value.value, null, 2)
  
  // 根据值的类型设置默认类型
  if (contextMenuData.value.value === null) {
    editForm.value.type = 'null'
  } else if (typeof contextMenuData.value.value === 'number') {
    editForm.value.type = 'number'
  } else if (typeof contextMenuData.value.value === 'boolean') {
    editForm.value.type = 'boolean'
  } else if (Array.isArray(contextMenuData.value.value)) {
    editForm.value.type = 'array'
  } else if (typeof contextMenuData.value.value === 'object') {
    editForm.value.type = 'object'
  } else {
    editForm.value.type = 'string'
  }
  
  showEditDialog.value = true
}

// 更新JSON数据中的值
function updateJsonValue(path: (string|number)[], newValue: any) {
  if (!parsedJson.value || !path.length) return
  
  let current = parsedJson.value
  const lastIndex = path.length - 1
  
  // 遍历到父节点
  for (let i = 0; i < lastIndex; i++) {
    current = current[path[i]]
    if (current === undefined || current === null) {
      console.error('路径无效:', path)
      return
    }
  }
  
  // 更新值
  current[path[lastIndex]] = newValue
  
  // 更新inputJson以保持同步
  inputJson.value = JSON.stringify(parsedJson.value, null, 2)
  
  // 保存到历史记录
  addToHistory(parsedJson.value)
}

const searchText = ref('')

// 递归过滤JSON，返回只包含匹配key或value的树
function filterJsonTree(data: any, keyword: string): any {
  if (!keyword) return data
  if (data === null || data === undefined) return null
  const kw = keyword.toLowerCase()
  
  if (Array.isArray(data)) {
    // 处理数组：保持数组结构，只显示匹配的元素
    const filteredArr = data
      .map((item, index) => {
        const filteredItem = filterJsonTree(item, keyword)
        return filteredItem !== null && filteredItem !== undefined ? filteredItem : null
      })
      .filter(item => item !== null && item !== undefined)
    return filteredArr.length > 0 ? filteredArr : null
  } else if (typeof data === 'object') {
    let matched = false
    const result: any = {}
    
    for (const [k, v] of Object.entries(data)) {
      // key 匹配
      if (k.toLowerCase().includes(kw)) {
        result[k] = v
        matched = true
        continue
      }
      
      // value 匹配
      if (typeof v === 'string' && v.toLowerCase().includes(kw)) {
        result[k] = v
        matched = true
        continue
      }
      if (typeof v === 'number' && v.toString().includes(kw)) {
        result[k] = v
        matched = true
        continue
      }
      if (typeof v === 'boolean' && v.toString().toLowerCase().includes(kw)) {
        result[k] = v
        matched = true
        continue
      }
      
      // 递归子节点
      const child = filterJsonTree(v, keyword)
      if (child !== null && child !== undefined) {
        // 检查子节点是否有内容
        const hasContent = Array.isArray(child) ? child.length > 0 : 
                          typeof child === 'object' ? Object.keys(child).length > 0 : true
        if (hasContent) {
          result[k] = child
          matched = true
        }
      }
    }
    return matched ? result : null
  } else {
    // 基本类型直接匹配
    if (typeof data === 'string' && data.toLowerCase().includes(kw)) return data
    if (typeof data === 'number' && data.toString().includes(kw)) return data
    if (typeof data === 'boolean' && data.toString().toLowerCase().includes(kw)) return data
    return null
  }
}

const filteredJson = computed(() => {
  if (!searchText.value) return parsedJson.value
  return filterJsonTree(parsedJson.value, searchText.value)
})
</script>

<style scoped>
/**** 只保留结构相关样式，布局交给全局和 assets 样式文件 ****/
</style> 