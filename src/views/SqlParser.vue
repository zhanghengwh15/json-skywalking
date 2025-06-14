<template>
  <div class="sql-parser">
    <div class="header">
      <h1>SQL 参数解析工具</h1>
      <p>输入带参数的SQL语句，自动解析并替换为完整的SQL</p>
    </div>
    
    <div class="content">
      <div class="input-section">
        <div class="section-header">
          <h3>输入SQL语句和参数</h3>
          <div class="actions">
            <button @click="parseSql" :disabled="loading" class="parse-btn">
              {{ loading ? '解析中...' : '解析 SQL' }}
            </button>
            <button @click="clearInput" class="clear-btn">清空</button>
            <button @click="loadExample" class="example-btn">示例</button>
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
    
    <div class="info-panel" v-if="parseInfo">
      <div class="info-grid">
        <div class="info-item">
          <h4>参数信息</h4>
          <div class="params-list">
            <div v-for="(param, index) in parseInfo.parameters" :key="index" class="param-item">
              <span class="param-index">{{ index + 1 }}</span>
              <span class="param-value">{{ param.value }}</span>
              <span class="param-type">{{ param.type }}</span>
            </div>
          </div>
        </div>
        <div class="info-item">
          <h4>统计信息</h4>
          <div class="stat-item">
            <span>参数数量：</span>
            <span>{{ parseInfo.paramCount }}</span>
          </div>
          <div class="stat-item">
            <span>占位符数量：</span>
            <span>{{ parseInfo.placeholderCount }}</span>
          </div>
          <div class="stat-item">
            <span>SQL长度：</span>
            <span>{{ parseInfo.sqlLength }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

// 响应式数据
const inputText = ref('')
const parsedSql = ref('')
const error = ref('')
const loading = ref(false)
const isFormatted = ref(false)
const parseInfo = ref<any>(null)

// 格式化后的SQL
const formattedSql = computed(() => {
  if (!parsedSql.value) return ''
  return isFormatted.value ? formatSqlString(parsedSql.value) : parsedSql.value
})

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
  
  // 确保SQL以SELECT开头
  if (!sqlBuilder.toLowerCase().startsWith('select')) {
    const selectIndex = sqlBuilder.toLowerCase().indexOf('select')
    if (selectIndex > 0) {
      sqlBuilder = 'SELECT ' + sqlBuilder.substring(selectIndex + 6)
    } else {
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
  parseInfo.value = null
  
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
    
    // 统计占位符数量
    const placeholderCount = (sqlPart.match(/\?/g) || []).length
    
    parsedSql.value = filledSql
    parseInfo.value = {
      parameters,
      paramCount: parameters.length,
      placeholderCount,
      sqlLength: filledSql.length
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
  parsedSql.value = ''
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
  if (!parsedSql.value) return
  
  try {
    await navigator.clipboard.writeText(parsedSql.value)
    alert('SQL已复制到剪贴板！')
  } catch (err) {
    // 降级方案
    const textArea = document.createElement('textarea')
    textArea.value = parsedSql.value
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
    alert('SQL已复制到剪贴板！')
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

// 加载示例
const loadExample = () => {
  inputText.value = `SELECT u.id, u.name, u.email, u.created_at 
FROM users u 
WHERE u.status = ? 
  AND u.created_at >= ? 
  AND u.department_id = ? 
  AND u.name LIKE ?
ORDER BY u.created_at DESC
LIMIT ?
db.sql.parameters: [1, 2024-01-01, 100, 张三, 10]`
}
</script>

<style scoped>
.sql-parser {
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

.clear-btn,
.copy-btn,
.format-btn,
.example-btn {
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
.example-btn:hover {
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
}

.sql-output {
  flex: 1;
  padding: 20px;
  background: #1e1e1e;
  overflow: auto;
  color: #d4d4d4;
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

.info-panel {
  margin-top: 20px;
  padding: 20px;
  background: #252526;
  border-radius: 8px;
  border-left: 4px solid #42b983;
  border: 1px solid #3c3c3c;
}

.info-grid {
  display: grid;
  grid-template-columns: 2fr 1fr;
  gap: 30px;
}

.info-item h4 {
  margin: 0 0 15px 0;
  color: #ffffff;
}

.params-list {
  max-height: 150px;
  overflow-y: auto;
}

.param-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 0;
  border-bottom: 1px solid #3c3c3c;
}

.param-item:last-child {
  border-bottom: none;
}

.param-index {
  background: #42b983;
  color: white;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: bold;
}

.param-value {
  flex: 1;
  font-family: 'Courier New', monospace;
  background: #2d2d30;
  padding: 4px 8px;
  border-radius: 3px;
  font-size: 13px;
  color: #d4d4d4;
}

.param-type {
  background: #6c757d;
  color: white;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: bold;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
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
</style> 