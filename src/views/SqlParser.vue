<template>
  <div class="sql-parser">
    <div class="header">
      <h1>SQL 参数解析工具</h1>
      <p>使用 Ctrl+V 直接粘贴SQL，自动解析并格式化</p>
    </div>
    
    <div class="content">
      <div class="input-section">
        <div class="section-header">
          <h3>输入SQL语句和参数</h3>
          <div class="actions">
            <button @click="parseSql" :disabled="loading" class="parse-btn">
              {{ loading ? '解析中...' : '解析 SQL' }}
            </button>
            <button @click="toggleHistory" class="history-btn">
              历史记录 ({{ historyList.length }})
            </button>
          </div>
        </div>
        <textarea
          v-model="inputText"
          placeholder="请输入格式：SQL语句&#10;db.sql.parameters: [参数1, 参数2, ...]"
          class="sql-input"
          @input="resetError"
        ></textarea>
      </div>
      
      <div class="output-section">
        <div class="section-header">
          <h3>解析结果</h3>
          <div class="actions">
            <button @click="copyToClipboard" :disabled="!parsedSql" class="copy-btn">
              复制SQL
            </button>
            <button @click="formatSql" :disabled="!parsedSql" class="format-btn">
              格式化
            </button>
          </div>
        </div>
        <div class="sql-output">
          <div v-if="error" class="error-message">
            <h4>解析错误：</h4>
            <p>{{ error }}</p>
          </div>
          <div v-else-if="parsedSql" class="sql-display">
            <pre>{{ formattedSql }}</pre>
          </div>
          <div v-else class="placeholder">
            解析后的SQL将在这里显示...
          </div>
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
              {{ item.sql.length > 100 ? item.sql.slice(0, 100) + '...' : item.sql }}
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
    
    <!-- 美化提示框 -->
    <div v-if="showToast" class="toast-overlay">
      <div class="toast-container" :class="toastType">
        <div class="toast-icon">
          <span v-if="toastType === 'success'">✅</span>
          <span v-else-if="toastType === 'error'">❌</span>
          <span v-else>ℹ️</span>
        </div>
        <div class="toast-message">{{ toastMessage }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onActivated, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { isValidJson } from '../utils/jsonUtils'

const router = useRouter()

// 响应式数据
const inputText = ref('')
const parsedSql = ref('')
const error = ref('')
const loading = ref(false)
const isFormatted = ref(false)

// 格式化后的SQL
const formattedSql = computed(() => {
  if (!parsedSql.value) return ''
  return isFormatted.value ? formatSqlString(parsedSql.value) : parsedSql.value
})

// 历史记录类型定义
interface SqlHistoryItem {
  sql: string
  formatted_sql: string
  timestamp: number
  hash: string
}

const historyList = ref<SqlHistoryItem[]>([])
const showHistory = ref(false)

// 提示框相关
const showToast = ref(false)
const toastMessage = ref('')
const toastType = ref<'success' | 'error' | 'info'>('info')

// 生成哈希
function generateHash(sql: string): string {
  // 使用简单的哈希算法
  let hash = 0
  for (let i = 0; i < sql.length; i++) {
    const char = sql.charCodeAt(i)
    hash = ((hash << 5) - hash) + char
    hash = hash & hash // 转换为32位整数
  }
  return Math.abs(hash).toString(36).slice(0, 16)
}

// 保存历史记录到文件
async function saveHistoryToFile() {
  try {
    await invoke('save_sql_history', { history: historyList.value })
    console.log('SQL历史记录已保存到文件')
  } catch (err) {
    console.error('保存SQL历史记录失败:', err)
  }
}

// 从文件加载历史记录
async function loadHistoryFromFile() {
  try {
    console.log('开始从文件加载SQL历史记录...');
    const history = await invoke<SqlHistoryItem[]>('load_sql_history')
    historyList.value = history || [] // 确保返回 null 或 undefined 时不会报错
    console.log(`成功加载 ${history.length} 条SQL历史记录。`)
  } catch (err) {
    console.error('加载SQL历史记录失败:', err)
    historyList.value = [] // 加载失败时清空列表
  }
}

// 添加到历史记录
function addToHistory(sql: string, formatted_sql: string) {
  const hash = generateHash(sql)
  
  // 检查是否已存在相同的SQL
  const existingIndex = historyList.value.findIndex(item => item.hash === hash)
  
  if (existingIndex !== -1) {
    // 如果已存在，更新时间戳
    historyList.value[existingIndex].timestamp = Date.now()
    // 重新排序
    historyList.value.sort((a, b) => b.timestamp - a.timestamp)
    // 保存到文件
    saveHistoryToFile()
    return
  }
  
  // 如果不存在，创建新记录
  const historyItem: SqlHistoryItem = {
    sql,
    formatted_sql,
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

// 从历史记录加载
function loadFromHistory(item: SqlHistoryItem) {
  inputText.value = item.sql
  parsedSql.value = item.formatted_sql
  error.value = ''
  showHistory.value = false
  isFormatted.value = true
  showToastMessage('已加载历史记录', 'success')
}

// 移除历史记录
function removeFromHistory(index: number) {
  historyList.value.splice(index, 1)
  saveHistoryToFile()
  showToastMessage('已删除历史记录', 'info')
}

// 清空历史记录
function clearHistory() {
  historyList.value = []
  saveHistoryToFile()
  showToastMessage('历史记录已清空', 'info')
}

// 切换历史记录显示
function toggleHistory() {
  showHistory.value = !showHistory.value
}

// 格式化时间
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

// JavaScript版本的参数解析逻辑
const parseParameters = (arrStr: string): Array<{value: any, type: string}> => {
  // 去除方括号并分割元素
  const cleanStr = arrStr.replace(/[\[\]]/g, '').trim()
  if (!cleanStr) return []
  
  const rawItems = cleanStr.split(',').map(item => item.trim())
  const result: Array<{value: any, type: string}> = []
  
  for (const item of rawItems) {
    const trimmedItem = item.trim()
    
    // 判断数据类型
    if (isDateString(trimmedItem)) {
      result.push({ value: `'${trimmedItem}'`, type: 'DATE' })
    } else if (containsLetters(trimmedItem)) {
      // 包含字母，作为字符串处理
      result.push({ value: `'${trimmedItem}'`, type: 'STRING' })
    } else if (isNumber(trimmedItem)) {
      result.push({ value: trimmedItem, type: 'NUMBER' })
    } else {
      // 默认作为字符串处理
      result.push({ value: `'${trimmedItem}'`, type: 'STRING' })
    }
  }
  
  return result
}

// 替换SQL中的参数
const replaceParameters = (sql: string, parameters: Array<{value: any, type: string}>): string => {
  let sqlBuilder = sql.trim()
  
  // 确保SQL以SELECT、UPDATE或INSERT开头
  const lowerSql = sqlBuilder.toLowerCase()
  if (!lowerSql.startsWith('select') && !lowerSql.startsWith('update') && !lowerSql.startsWith('insert')) {
    // 查找第一个SQL关键字
    const selectIndex = lowerSql.indexOf('select')
    const updateIndex = lowerSql.indexOf('update')
    const insertIndex = lowerSql.indexOf('insert')
    
    let keywordIndex = -1
    let keyword = ''
    
    // 找到第一个出现的关键字
    if (selectIndex >= 0) {
      keywordIndex = selectIndex
      keyword = 'SELECT'
    }
    if (updateIndex >= 0 && (keywordIndex === -1 || updateIndex < keywordIndex)) {
      keywordIndex = updateIndex
      keyword = 'UPDATE'
    }
    if (insertIndex >= 0 && (keywordIndex === -1 || insertIndex < keywordIndex)) {
      keywordIndex = insertIndex
      keyword = 'INSERT'
    }
    
    if (keywordIndex > 0) {
      // 如果关键字不在开头，则从关键字开始截取
      sqlBuilder = keyword + ' ' + sqlBuilder.substring(keywordIndex + keyword.length)
    } else {
      // 如果没找到关键字，默认添加SELECT
      sqlBuilder = 'SELECT ' + sqlBuilder
    }
  }
  
  // 分割SQL，按?占位符
  const parts = sqlBuilder.split('?')
  let result = ''
  
  for (let i = 0; i < parts.length; i++) {
    result += parts[i]
    if (i < parameters.length) {
      result += parameters[i].value
    }
  }
  
  return result
}

// 工具函数：判断是否为日期字符串
const isDateString = (str: string): boolean => {
  // 简单的日期格式判断
  const datePatterns = [
    /^\d{4}-\d{2}-\d{2}$/, // YYYY-MM-DD
    /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$/, // YYYY-MM-DD HH:mm:ss
    /^\d{4}\/\d{2}\/\d{2}$/, // YYYY/MM/DD
  ]
  return datePatterns.some(pattern => pattern.test(str))
}

// 工具函数：判断是否包含字母
const containsLetters = (str: string): boolean => {
  return /[a-zA-Z]/.test(str)
}

// 工具函数：判断是否为数字
const isNumber = (str: string): boolean => {
  return !isNaN(Number(str)) && !isNaN(parseFloat(str))
}

// 主要的SQL解析函数
const parseSql = async () => {
  if (!inputText.value.trim()) {
    error.value = '请输入SQL语句和参数'
    return
  }
  
  loading.value = true
  error.value = ''
  parsedSql.value = ''
  
  try {
    // 模拟异步处理
    await new Promise(resolve => setTimeout(resolve, 100))
    
    // 按照"db.sql.parameters:"分割
    const parts = inputText.value.split('db.sql.parameters:')
    
    if (parts.length !== 2) {
      throw new Error('格式不正确，应该包含 "db.sql.parameters:" 分隔符')
    }
    
    const sqlPart = parts[0].trim()
    const paramsPart = parts[1].trim()
    
    if (!sqlPart) {
      throw new Error('SQL语句不能为空')
    }
    
    // 解析参数
    const parameters = parseParameters(paramsPart)
    
    // 替换参数
    const filledSql = replaceParameters(sqlPart, parameters)
    
    parsedSql.value = filledSql
    
    // 添加到历史记录
    if (isFormatted.value) {
      addToHistory(inputText.value, formattedSql.value)
    } else {
      addToHistory(inputText.value, filledSql)
    }
    
    showToastMessage('SQL解析成功！', 'success')
    
  } catch (err) {
    error.value = err instanceof Error ? err.message : '解析失败'
  } finally {
    loading.value = false
  }
}

// 重置错误
const resetError = () => {
  error.value = ''
}

// 显示提示框
const showToastMessage = (message: string, type: 'success' | 'error' | 'info' = 'info') => {
  toastMessage.value = message
  toastType.value = type
  showToast.value = true
  
  // 3秒后自动消失
  setTimeout(() => {
    showToast.value = false
  }, 3000)
}

// 复制到剪贴板
const copyToClipboard = async () => {
  if (!parsedSql.value) return
  
  try {
    await navigator.clipboard.writeText(parsedSql.value)
    showToastMessage('SQL已复制到剪贴板！', 'success')
  } catch (err) {
    // 降级方案
    const textArea = document.createElement('textarea')
    textArea.value = parsedSql.value
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
    showToastMessage('SQL已复制到剪贴板！', 'success')
  }
}

// 格式化SQL
const formatSql = () => {
  isFormatted.value = !isFormatted.value
}

// SQL格式化函数
const formatSqlString = (sql: string): string => {
  // 简单的SQL格式化
  let formatted = sql
    .replace(/\bSELECT\b/gi, '\nSELECT')
    .replace(/\bFROM\b/gi, '\nFROM')
    .replace(/\bWHERE\b/gi, '\nWHERE')
    .replace(/\bAND\b/gi, '\n  AND')
    .replace(/\bOR\b/gi, '\n  OR')
    .replace(/\bORDER BY\b/gi, '\nORDER BY')
    .replace(/\bGROUP BY\b/gi, '\nGROUP BY')
    .replace(/\bHAVING\b/gi, '\nHAVING')
    .replace(/\bLIMIT\b/gi, '\nLIMIT')
    .replace(/\bJOIN\b/gi, '\nJOIN')
    .replace(/\bLEFT JOIN\b/gi, '\nLEFT JOIN')
    .replace(/\bRIGHT JOIN\b/gi, '\nRIGHT JOIN')
    .replace(/\bINNER JOIN\b/gi, '\nINNER JOIN')
  
  return formatted.replace(/^\n/, '') // 移除开头的换行
}

// 自动获取剪贴板内容并处理
const autoProcessClipboard = async () => {
  try {
    const clipboardText = await invoke<string>('get_clipboard')
    if (!clipboardText || !clipboardText.trim()) return
    
    // 检测是否为JSON格式
    if (isValidJson(clipboardText)) {
      // 如果是JSON，保存到localStorage并跳转到JsonParser
      localStorage.setItem('tempJsonData', clipboardText)
      router.push('/json-parser')
      return
    }
    
    // 设置输入内容
    inputText.value = clipboardText
    // 自动解析
    await parseSql()
    // 自动格式化
    isFormatted.value = true
    // 自动复制格式化后的结果
    if (parsedSql.value) {
      await navigator.clipboard.writeText(formattedSql.value)
      showToastMessage('已自动复制格式化后的SQL到剪贴板', 'success')
    }
  } catch (err) {
    console.error('处理剪贴板内容失败:', err)
    showToastMessage('处理剪贴板内容失败', 'error')
  }
}

// 键盘事件处理
const handleKeydown = async (event: KeyboardEvent) => {
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0
  const isModifierPressed = isMac ? event.metaKey : event.ctrlKey
  
  // Command+V (Mac) 或 Ctrl+V (Windows/Linux)
  if (isModifierPressed && event.key.toLowerCase() === 'v') {
    event.preventDefault()
    event.stopPropagation()
    await autoProcessClipboard()
  }
}

// 生命周期钩子
onMounted(async () => {
  // 从文件加载历史记录
  await loadHistoryFromFile()
  document.addEventListener('keydown', handleKeydown)
})

onActivated(async () => {
  // 组件激活时自动处理剪贴板内容，并重新加载历史
  await loadHistoryFromFile()
  await autoProcessClipboard()
})

// 组件卸载时移除事件监听器
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.sql-parser {
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
  margin: 0 auto;
}

.header {
  text-align: center;
  margin-bottom: 20px;
  flex-shrink: 0;
  padding: 0 20px;
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

.section-header h3 {
  margin: 0;
  font-size: 16px;
}

.actions {
  display: flex;
  gap: 10px;
}

.parse-btn {
  background: #27ae60;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s ease;
}

.parse-btn:hover:not(:disabled) {
  background: #229954;
  transform: translateY(-1px);
}

.parse-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.copy-btn,
.format-btn {
  background: rgba(255, 255, 255, 0.2);
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.3);
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s ease;
}

.copy-btn:hover:not(:disabled),
.format-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.3);
}

.sql-input {
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

.sql-output {
  flex: 1;
  padding: 20px;
  background: #1e1e1e;
  overflow: auto;
  color: #d4d4d4;
  min-height: 0;
}

.sql-display pre {
  font-family: 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.5;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
  background: #2c3e50;
  color: #ecf0f1;
  padding: 15px;
  border-radius: 4px;
}

.placeholder {
  color: #6a6a6a;
  text-align: center;
  padding: 40px 20px;
  font-style: italic;
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

/* 响应式设计 */
@media (max-width: 768px) {
  .content {
    grid-template-columns: 1fr;
    grid-template-rows: 1fr 1fr;
  }
  
  .sql-parser {
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
}

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

.history-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.history-sidebar {
  position: fixed;
  top: 0;
  right: 0;
  width: 400px;
  height: 100vh;
  background: #252526;
  border-left: 1px solid #3c3c3c;
  display: flex;
  flex-direction: column;
  z-index: 1000;
}

.history-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.5);
  z-index: 999;
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  background: #2d2d30;
  border-bottom: 1px solid #3c3c3c;
}

.history-header h3 {
  margin: 0;
  color: #ffffff;
}

.close-btn {
  background: none;
  border: none;
  color: #cccccc;
  font-size: 20px;
  cursor: pointer;
  padding: 0;
}

.close-btn:hover {
  color: #ffffff;
}

.history-content {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.history-empty {
  color: #6a6a6a;
  text-align: center;
  padding: 40px 20px;
  font-style: italic;
}

.history-item {
  background: #1e1e1e;
  border: 1px solid #3c3c3c;
  border-radius: 4px;
  margin-bottom: 10px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.history-item:hover {
  border-color: #42b983;
  transform: translateX(-2px);
}

.history-preview {
  padding: 15px;
  font-family: 'Courier New', monospace;
  font-size: 13px;
  color: #d4d4d4;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 100px;
  overflow: hidden;
}

.history-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 15px;
  background: #2d2d30;
  border-top: 1px solid #3c3c3c;
}

.history-time {
  color: #a0a0a0;
  font-size: 12px;
}

.remove-btn {
  background: none;
  border: none;
  color: #f48771;
  cursor: pointer;
  font-size: 12px;
  padding: 4px 8px;
  border-radius: 3px;
}

.remove-btn:hover {
  background: rgba(244, 135, 113, 0.2);
}

.history-footer {
  padding: 15px;
  background: #2d2d30;
  border-top: 1px solid #3c3c3c;
}

.clear-all-btn {
  width: 100%;
  background: #e74c3c;
  color: white;
  border: none;
  padding: 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.3s ease;
}

.clear-all-btn:hover:not(:disabled) {
  background: #c0392b;
}

.clear-all-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* 提示框样式 */
.toast-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 2000;
  pointer-events: none;
}

.toast-container {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 24px;
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  animation: toastSlideIn 0.3s ease-out;
  pointer-events: auto;
  min-width: 280px;
  max-width: 400px;
}

.toast-container.success {
  background: linear-gradient(135deg, #27ae60, #2ecc71);
  color: white;
}

.toast-container.error {
  background: linear-gradient(135deg, #e74c3c, #c0392b);
  color: white;
}

.toast-container.info {
  background: linear-gradient(135deg, #3498db, #2980b9);
  color: white;
}

.toast-icon {
  font-size: 20px;
  flex-shrink: 0;
}

.toast-message {
  font-size: 14px;
  font-weight: 500;
  line-height: 1.4;
  flex: 1;
}

@keyframes toastSlideIn {
  from {
    opacity: 0;
    transform: translateY(-20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes toastSlideOut {
  from {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
  to {
    opacity: 0;
    transform: translateY(-20px) scale(0.95);
  }
}
</style> 