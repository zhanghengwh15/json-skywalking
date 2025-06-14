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
          </div>
        </div>
        <textarea
          v-model="inputJson"
          placeholder="请输入要解析的JSON数据..."
          class="json-input"
          @input="resetError"
        ></textarea>
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
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

// 响应式数据
const inputJson = ref('')
const parsedJson = ref<any>(null)
const error = ref('')
const loading = ref(false)
const isCompact = ref(false)

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
    // 这里可以添加提示信息
    alert('已复制到剪贴板！')
  } catch (err) {
    // 降级方案
    const textArea = document.createElement('textarea')
    textArea.value = jsonString
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
    alert('已复制到剪贴板！')
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

// 示例JSON数据
const loadExample = () => {
  inputJson.value = `{
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

// 页面加载时加载示例
loadExample()
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
.compact-btn {
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
.compact-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.3);
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

.info-panel {
  margin-top: 20px;
  padding: 15px;
  background: #f0f2f5;
  border-radius: 8px;
  border-left: 4px solid #42b983;
}

.stats h4 {
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
}
</style> 