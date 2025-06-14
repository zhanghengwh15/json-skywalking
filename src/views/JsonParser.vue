<template>
  <div class="json-parser">
    <div class="header">
      <h1>JSON 解析与格式化工具</h1>
      <p>在左侧输入JSON，点击解析按钮格式化并在右侧显示</p>
    </div>
    
    <div class="content">
      <div class="input-section">
        <div class="section-header">
          <h3>输入JSON</h3>
          <div class="actions">
            <button @click="parseJson" :disabled="loading" class="parse-btn">
              {{ loading ? '解析中...' : '解析 JSON' }}
            </button>
            <button @click="clearInput" class="clear-btn">清空</button>
            <button @click="getClipboardContent" class="clipboard-btn" :title="'获取剪贴板 (全局快捷键: ⌘⇧G)'">
              获取剪贴板
              <span class="shortcut-hint">⌘⇧G</span>
              <span class="global-indicator">🌍</span>
            </button>
          </div>
        </div>
        <textarea
          v-model="inputJson"
          placeholder="请输入要解析的JSON数据..."
          class="json-input"
          @input="resetError"
        ></textarea>
        <div v-if="clipboardStatus" class="clipboard-status" :class="clipboardStatus.type">
          {{ clipboardStatus.message }}
        </div>
      </div>
      
      <div class="output-section">
        <div class="section-header">
          <h3>格式化结果</h3>
          <div class="actions">
            <button @click="copyToClipboard" :disabled="!parsedJson" class="copy-btn">
              复制结果
            </button>
            <button @click="compactJson" :disabled="!parsedJson" class="compact-btn">
              压缩格式
            </button>
          </div>
        </div>
        <div class="json-output">
          <div v-if="error" class="error-message">
            <h4>解析错误：</h4>
            <p>{{ error }}</p>
          </div>
          <div v-else-if="parsedJson" class="json-display" v-html="highlightedJson"></div>
          <div v-else class="placeholder">
            解析后的JSON将在这里显示...
          </div>
        </div>
      </div>
    </div>
    
    <div class="info-panel">
      <div class="stats" v-if="jsonStats">
        <h4>JSON 信息</h4>
        <div class="stat-item">
          <span>类型：</span>
          <span>{{ jsonStats.type }}</span>
        </div>
        <div class="stat-item">
          <span>键的数量：</span>
          <span>{{ jsonStats.keyCount }}</span>
        </div>
        <div class="stat-item">
          <span>字符长度：</span>
          <span>{{ jsonStats.length }}</span>
        </div>
      </div>
      
      <div class="keyboard-shortcuts">
        <h4>快捷键</h4>
        <div class="shortcut-item">
          <kbd>⌘</kbd> + <kbd>⇧</kbd> + <kbd>G</kbd>
          <span>全局获取剪贴板 🌍</span>
        </div>
        <div class="shortcut-note">
          💡 即使软件在后台运行也可以使用
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onActivated, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

// 响应式数据
const inputJson = ref('')
const parsedJson = ref<any>(null)
const error = ref('')
const loading = ref(false)
const isCompact = ref(false)
const clipboardStatus = ref<{type: 'success' | 'error' | 'info', message: string} | null>(null)

// JSON统计信息
const jsonStats = computed(() => {
  if (!parsedJson.value) return null
  
  const getType = (obj: any): string => {
    if (Array.isArray(obj)) return 'Array'
    if (obj === null) return 'null'
    return typeof obj === 'object' ? 'Object' : typeof obj
  }
  
  const countKeys = (obj: any): number => {
    if (Array.isArray(obj)) return obj.length
    if (typeof obj === 'object' && obj !== null) {
      return Object.keys(obj).length
    }
    return 0
  }
  
  return {
    type: getType(parsedJson.value),
    keyCount: countKeys(parsedJson.value),
    length: JSON.stringify(parsedJson.value).length
  }
})

// 高亮显示的JSON
const highlightedJson = computed(() => {
  if (!parsedJson.value) return ''
  
  const jsonString = isCompact.value 
    ? JSON.stringify(parsedJson.value)
    : JSON.stringify(parsedJson.value, null, 2)
  
  return syntaxHighlight(jsonString)
})

// 检查字符串是否为有效JSON
const isValidJson = (str: string): boolean => {
  try {
    JSON.parse(str)
    return true
  } catch {
    return false
  }
}

// 键盘事件处理器（本地快捷键作为备用）
const handleKeydown = (event: KeyboardEvent) => {
  // 检测 Command+Shift+G (Mac) 或 Ctrl+Shift+G (Windows/Linux)
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0
  const isModifierPressed = isMac ? event.metaKey : event.ctrlKey
  
  if (isModifierPressed && event.shiftKey && event.key.toLowerCase() === 'g') {
    event.preventDefault()
    event.stopPropagation()
    getClipboardContent()
  }
}

// 处理全局快捷键事件
const setupGlobalShortcutListeners = async () => {
  try {
    // 监听全局快捷键触发的JSON格式剪贴板事件
    await listen('global-clipboard-json', (event) => {
      const clipboardText = event.payload as string
      inputJson.value = clipboardText
      showClipboardStatus('success', '🌍 全局快捷键检测到JSON格式，已自动填入')
      
      // 自动解析JSON
      setTimeout(() => {
        parseJson()
      }, 100)
    })
    
    // 监听全局快捷键触发的非JSON格式剪贴板事件
    await listen('global-clipboard-not-json', (event) => {
      showClipboardStatus('error', '🌍 全局快捷键: 剪贴板内容不是有效的JSON格式')
    })
    
    console.log('Global shortcut event listeners registered')
  } catch (err) {
    console.error('Failed to setup global shortcut listeners:', err)
  }
}

// 获取剪贴板内容
const getClipboardContent = async () => {
  try {
    const clipboardText = await invoke<string>('get_clipboard')
    
    if (!clipboardText || !clipboardText.trim()) {
      showClipboardStatus('info', '剪贴板为空')
      return
    }
    
    // 检查是否为有效JSON
    if (isValidJson(clipboardText)) {
      inputJson.value = clipboardText
      showClipboardStatus('success', '检测到JSON格式，已自动填入 (本地快捷键)')
      // 自动解析JSON
      setTimeout(() => {
        parseJson()
      }, 100)
    } else {
      showClipboardStatus('error', '剪贴板内容不是有效的JSON格式')
    }
  } catch (err) {
    console.error('获取剪贴板失败:', err)
    showClipboardStatus('error', '获取剪贴板失败: ' + (err as Error).message)
  }
}

// 显示剪贴板状态
const showClipboardStatus = (type: 'success' | 'error' | 'info', message: string) => {
  clipboardStatus.value = { type, message }
  setTimeout(() => {
    clipboardStatus.value = null
  }, 4000) // 增加到4秒，给用户更多时间看到全局快捷键提示
}

// 自动检查剪贴板（页面加载时）
const autoCheckClipboard = async () => {
  try {
    const clipboardText = await invoke<string>('get_clipboard')
    
    if (clipboardText && clipboardText.trim() && isValidJson(clipboardText)) {
      // 如果输入框为空或只有示例数据，则自动填入
      if (!inputJson.value.trim() || inputJson.value === getExampleJson()) {
        inputJson.value = clipboardText
        showClipboardStatus('success', '自动检测到剪贴板中的JSON格式数据')
        // 自动解析JSON
        setTimeout(() => {
          parseJson()
        }, 100)
      }
    }
  } catch (err) {
    // 静默处理错误，不影响正常使用
    console.log('自动检查剪贴板失败:', err)
  }
}

// 解析JSON
const parseJson = async () => {
  if (!inputJson.value.trim()) {
    error.value = '请输入JSON数据'
    return
  }
  
  loading.value = true
  error.value = ''
  
  try {
    // 模拟异步解析（实际上JSON.parse是同步的）
    await new Promise(resolve => setTimeout(resolve, 100))
    
    parsedJson.value = JSON.parse(inputJson.value)
    isCompact.value = false
  } catch (err) {
    error.value = err instanceof Error ? err.message : '无效的JSON格式'
    parsedJson.value = null
  } finally {
    loading.value = false
  }
}

// 清空输入
const clearInput = () => {
  inputJson.value = ''
  parsedJson.value = null
  error.value = ''
  clipboardStatus.value = null
}

// 重置错误
const resetError = () => {
  error.value = ''
}

// 复制到剪贴板
const copyToClipboard = async () => {
  if (!parsedJson.value) return
  
  const jsonString = isCompact.value 
    ? JSON.stringify(parsedJson.value)
    : JSON.stringify(parsedJson.value, null, 2)
  
  try {
    await navigator.clipboard.writeText(jsonString)
    showClipboardStatus('success', '已复制到剪贴板！')
  } catch (err) {
    // 降级方案
    const textArea = document.createElement('textarea')
    textArea.value = jsonString
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
    showClipboardStatus('success', '已复制到剪贴板！')
  }
}

// 压缩JSON格式
const compactJson = () => {
  isCompact.value = !isCompact.value
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

// 获取示例JSON数据
const getExampleJson = (): string => {
  return `{
  "name": "张三",
  "age": 30,
  "isStudent": false,
  "address": {
    "city": "北京",
    "district": "海淀区",
    "street": "中关村大街1号"
  },
  "hobbies": ["编程", "阅读", "运动"],
  "contact": {
    "email": "zhangsan@example.com",
    "phone": "13800138000"
  },
  "metadata": null
}`
}

// 示例JSON数据
const loadExample = () => {
  inputJson.value = getExampleJson()
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

// 如果没有剪贴板内容，则加载示例
setTimeout(() => {
  if (!inputJson.value.trim()) {
    loadExample()
  }
}, 200)
</script>

<style scoped>
.json-parser {
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
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
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
  background: #42b983;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s ease;
}

.parse-btn:hover:not(:disabled) {
  background: #369870;
  transform: translateY(-1px);
}

.parse-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.clear-btn,
.copy-btn,
.compact-btn,
.clipboard-btn {
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

.clear-btn:hover,
.copy-btn:hover:not(:disabled),
.compact-btn:hover:not(:disabled),
.clipboard-btn:hover {
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

.json-input {
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

.json-output {
  flex: 1;
  padding: 20px;
  background: #f8f9fa;
  overflow: auto;
}

.json-display {
  font-family: 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
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

.clipboard-status {
  padding: 10px 20px;
  font-size: 14px;
  border-top: 1px solid #ddd;
  animation: fadeInOut 4s ease-in-out;
}

.clipboard-status.success {
  background: #d4edda;
  color: #155724;
  border-left: 4px solid #28a745;
}

.clipboard-status.error {
  background: #f8d7da;
  color: #721c24;
  border-left: 4px solid #dc3545;
}

.clipboard-status.info {
  background: #d1ecf1;
  color: #0c5460;
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
  background: #f0f2f5;
  border-radius: 8px;
  border-left: 4px solid #42b983;
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 20px;
}

.stats h4,
.keyboard-shortcuts h4 {
  margin: 0 0 15px 0;
  color: #2c3e50;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  padding: 5px 0;
  border-bottom: 1px solid #ddd;
}

.stat-item:last-child {
  border-bottom: none;
}

.stat-item span:first-child {
  font-weight: 500;
  color: #666;
}

.stat-item span:last-child {
  color: #2c3e50;
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
  color: #666;
  font-size: 14px;
}

.shortcut-item kbd {
  background: #fff;
  border: 1px solid #ccc;
  border-radius: 3px;
  padding: 2px 6px;
  font-size: 12px;
  color: #333;
  box-shadow: 0 1px 2px rgba(0,0,0,0.1);
  font-family: monospace;
}

.shortcut-item span {
  color: #2c3e50;
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

/* JSON语法高亮样式 */
:deep(.json-key) {
  color: #e74c3c;
  font-weight: bold;
}

:deep(.json-string) {
  color: #27ae60;
}

:deep(.json-number) {
  color: #3498db;
}

:deep(.json-boolean) {
  color: #9b59b6;
  font-weight: bold;
}

:deep(.json-null) {
  color: #95a5a6;
  font-weight: bold;
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
  
  .info-panel {
    grid-template-columns: 1fr;
  }
  
  .shortcut-hint {
    display: none;
  }
  
  .global-indicator {
    display: none;
  }
}
</style> 