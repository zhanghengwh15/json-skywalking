<script setup lang="ts">
import { ref } from "vue";
</script>

<template>
  <div id="app">
    <nav class="tab-navigation">
      <router-link to="/" class="tab-item">
        <span class="tab-icon">🏠</span>
        <span class="tab-text">首页</span>
      </router-link>
      <router-link to="/about" class="tab-item">
        <span class="tab-icon">ℹ️</span>
        <span class="tab-text">关于</span>
      </router-link>
      <router-link to="/json-parser" class="tab-item">
        <span class="tab-icon">🔧</span>
        <span class="tab-text">JSON解析</span>
      </router-link>
      <router-link to="/sql-parser" class="tab-item">
        <span class="tab-icon">🗃️</span>
        <span class="tab-text">SQL解析</span>
      </router-link>
    </nav>
    <main class="main-content">
      <router-view></router-view>
    </main>
  </div>
</template>

<style>
#app {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.tab-navigation {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 0;
  display: flex;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
}

.tab-item {
  display: flex;
  align-items: center;
  padding: 15px 25px;
  color: rgba(255, 255, 255, 0.8);
  text-decoration: none;
  font-weight: 500;
  transition: all 0.3s ease;
  border-bottom: 3px solid transparent;
  flex: 1;
  justify-content: center;
  gap: 8px;
}

.tab-item:hover {
  color: white;
  background-color: rgba(255, 255, 255, 0.1);
}

.tab-item.router-link-exact-active {
  color: white;
  border-bottom-color: #42b983;
  background-color: rgba(255, 255, 255, 0.15);
}

.tab-icon {
  font-size: 18px;
}

.tab-text {
  font-size: 14px;
}

.main-content {
  flex: 1;
  overflow-y: auto;
  background-color: #f8f9fa;
}

/* 响应式设计 */
@media (max-width: 480px) {
  .tab-item {
    flex-direction: column;
    padding: 10px 15px;
    gap: 4px;
  }
  
  .tab-text {
    font-size: 12px;
  }
}
</style>