<script setup lang="ts">
// 移除未使用的import
</script>

<template>
  <div id="app">
    <nav class="tab-navigation">
      <router-link to="/json-parser" class="tab-item">
        <span class="tab-icon">🔧</span>
        <span class="tab-text">JSON解析</span>
      </router-link>
      <router-link to="/sql-parser" class="tab-item">
        <span class="tab-icon">🗃️</span>
        <span class="tab-text">SQL解析</span>
      </router-link>
      <router-link to="/http-parser" class="tab-item">
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
/* 全局样式 - 确保页面填充整个窗口 */
html,
body,
#app {
  height: 100%;
  width: 100%;
  margin: 0;
  padding: 0;
  overflow: hidden; /* 防止出现不必要的滚动条 */
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
    'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
    sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* 全局 box-sizing 和 margin/padding reset */
*, *::before, *::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

/* 让 #app 使用 flex 布局 */
#app {
  display: flex;
  flex-direction: column;
}

.main-content {
  height: auto;
  min-height: 0;
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
}

/* 兼容所有页面根容器（如 .json-parser） */
.json-parser {
  height: 100%;
  min-height: 0;
  display: flex;
  flex-direction: column;
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

/* 响应式设计 - 针对不同屏幕尺寸的优化 */
@media (max-width: 1200px) {
  /* 中等屏幕 */
  .json-parser {
    max-width: 100% !important;
  }
}

@media (max-width: 768px) {
  /* 小屏幕 */
  .content {
    grid-template-columns: 1fr !important;
    gap: 10px !important;
  }
  
  .section-header {
    padding: 10px 15px !important;
  }
  
  .actions {
    flex-direction: column !important;
    gap: 5px !important;
  }
}

/* 高分辨率屏幕优化 */
@media (min-width: 1920px) {
  /* 4K或更大屏幕 */
  .json-parser {
    font-size: 16px;
  }
  
  .json-tree {
    font-size: 15px;
  }
  
  .section-header h3 {
    font-size: 18px;
  }
}
</style>