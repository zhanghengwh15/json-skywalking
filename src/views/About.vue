<template>
  <div class="about">
    <h1>关于我们</h1>
    <div class="content">
      <p>这是一个基于Tauri + Vue 3的桌面应用程序</p>
      <div class="features">
        <h2>主要特性</h2>
        <ul>
          <li>跨平台桌面应用</li>
          <li>Vue 3 + TypeScript</li>
          <li>Rust后端支持</li>
          <li>现代化UI界面</li>
        </ul>
      </div>
      
      <!-- 剪贴板功能区域 -->
      <div class="clipboard-section">
        <h2>剪贴板功能</h2>
        <button @click="getClipboard" :disabled="loading" class="clipboard-btn">
          {{ loading ? '获取中...' : '获取剪贴板内容' }}
        </button>
        <div v-if="clipboardContent" class="clipboard-content">
          <h3>剪贴板内容：</h3>
          <div class="clipboard-text">{{ clipboardContent }}</div>
        </div>
        <div v-if="error" class="error-message">
          <p>错误：{{ error }}</p>
        </div>
      </div>
      
      <div class="version-info">
        <p><strong>版本：</strong>v0.1.0</p>
        <p><strong>构建日期：</strong>{{ buildDate }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// 组件逻辑
const buildDate = ref(new Date().toLocaleDateString('zh-CN'))
const clipboardContent = ref<string>('')
const loading = ref(false)
const error = ref<string>('')

const getClipboard = async () => {
  loading.value = true
  error.value = ''
  clipboardContent.value = ''
  
  try {
    const result = await invoke<string>('get_clipboard')
    clipboardContent.value = result
  } catch (err) {
    error.value = err as string
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.about {
  padding: 40px;
  max-width: 800px;
  margin: 0 auto;
}

.content {
  text-align: left;
  line-height: 1.6;
}

.features {
  margin: 30px 0;
  padding: 20px;
  background-color: #f5f5f5;
  border-radius: 8px;
}

.features h2 {
  color: #42b983;
  margin-bottom: 15px;
}

.features ul {
  list-style-type: none;
  padding: 0;
}

.features li {
  padding: 8px 0;
  border-bottom: 1px solid #ddd;
}

.features li:before {
  content: "✓ ";
  color: #42b983;
  font-weight: bold;
}

.clipboard-section {
  margin: 30px 0;
  padding: 20px;
  background-color: #f8f9fa;
  border-radius: 8px;
  border-left: 4px solid #007bff;
}

.clipboard-section h2 {
  color: #007bff;
  margin-bottom: 15px;
}

.clipboard-btn {
  background: linear-gradient(135deg, #007bff, #0056b3);
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 6px;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.3s ease;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.clipboard-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(0,0,0,0.15);
}

.clipboard-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.clipboard-content {
  margin-top: 20px;
  padding: 15px;
  background-color: white;
  border: 1px solid #dee2e6;
  border-radius: 4px;
}

.clipboard-content h3 {
  margin: 0 0 10px 0;
  color: #495057;
  font-size: 14px;
}

.clipboard-text {
  background-color: #f8f9fa;
  padding: 12px;
  border-radius: 4px;
  font-family: 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.4;
  word-break: break-all;
  white-space: pre-wrap;
  border: 1px solid #e9ecef;
  max-height: 200px;
  overflow-y: auto;
}

.error-message {
  margin-top: 15px;
  padding: 12px;
  background-color: #f8d7da;
  color: #721c24;
  border: 1px solid #f1aeb5;
  border-radius: 4px;
}

.version-info {
  margin-top: 30px;
  padding: 15px;
  background-color: #e8f4f8;
  border-left: 4px solid #42b983;
}

.version-info p {
  margin: 5px 0;
}
</style> 