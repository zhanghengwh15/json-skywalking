<script setup lang="ts">
// 移除未使用的import
</script>

<template>
  <div id="app">
    <nav class="tab-navigation">
      <router-link to="/" class="tab-item">
        <span class="tab-icon">🏠</span>
        <span class="tab-text">首页</span>
      </router-link>
      <router-link to="/json-parser" class="tab-item">
        <span class="tab-icon">🔧</span>
        <span class="tab-text">JSON解析</span>
      </router-link>
      <router-link to="/sql-parser" class="tab-item">
        <span class="tab-icon">🗃️</span>
        <span class="tab-text">SQL解析</span>
      </router-link>
      <router-link to="/post-parser" class="tab-item">
        <span class="tab-icon">🌐</span>
        <span class="tab-text">HTTP解析</span>
      </router-link>
    </nav>
    <main class="main-content">
      <router-view></router-view>
    </main>
  </div>
</template>

<style>
html, body, #app {
  height: 100%;
  min-height: 0;
  margin: 0;
  padding: 0;
}

.tab-navigation {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 0;
  display: flex;
  box-shadow: 0 2px 10px rgba(0,0,0,0.1);
  flex-shrink: 0;
  z-index: 10;
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
  height: 60px;
  box-sizing: border-box;
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
  min-height: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  /* 不设置overflow，交由子页面决定 */
}

/* 添加全局样式 */
html, body {
  overflow: hidden;
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