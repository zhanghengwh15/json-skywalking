<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()

interface NavItem {
  path: string
  name: string
  icon: string
}

const navItems: NavItem[] = [
  { path: '/json-parser', name: 'JSON解析', icon: 'data_object' },
  { path: '/sql-parser', name: 'SQL解析', icon: 'storage' },
  { path: '/http-parser', name: 'HTTP解析', icon: 'http' },
  { path: '/cookie-bridge', name: 'Cookie信息', icon: 'cookie' },
  { path: '/task-branch-group', name: '任务分支', icon: 'account_tree' },
]

const currentNavName = computed(() => {
  const item = navItems.find(item => item.path === route.path)
  return item?.name || '开发工具箱'
})

const isActive = (path: string) => route.path === path

const sidebarCollapsed = ref(false)
function toggleSidebar() {
  sidebarCollapsed.value = !sidebarCollapsed.value
}
</script>

<template>
  <div id="app">
    <!-- SideNavBar -->
    <aside class="sidebar" :class="{ collapsed: sidebarCollapsed }">
      <div class="sidebar-header">
        <div v-if="!sidebarCollapsed" class="sidebar-brand">开发工具箱</div>
        <button class="sidebar-toggle" @click="toggleSidebar" title="收起/展开">
          <span class="material-icons">{{ sidebarCollapsed ? 'chevron_right' : 'chevron_left' }}</span>
        </button>
      </div>
      <nav class="sidebar-nav">
        <router-link
          v-for="item in navItems"
          :key="item.path"
          :to="item.path"
          :class="['nav-item', { active: isActive(item.path) }]"
        >
          <span class="nav-icon material-icons">{{ item.icon }}</span>
          <span v-if="!sidebarCollapsed" class="nav-text">{{ item.name }}</span>
        </router-link>
      </nav>
    </aside>
    <!-- Main Content -->

    <!-- Main Content -->
    <main class="main-content">
      <!-- TopAppBar -->
      <header class="topbar">
        <div class="topbar-left">
          <div class="breadcrumb">
            <span class="breadcrumb-text">{{ currentNavName }}</span>
          </div>
        </div>
        <div class="topbar-right">
          <span class="version-tag">v0.1.0</span>
        </div>
      </header>

      <!-- Page Content -->
      <div class="page-content">
        <router-view />
      </div>
    </main>
  </div>
</template>

<style>
/* ========== 全局重置与主题变量 ========== */
html, body, #app {
  height: 100%;
  width: 100%;
  margin: 0;
  padding: 0;
  overflow: hidden;
}

:root {
  --bg-primary: #131315;
  --bg-secondary: #1b1b1d;
  --bg-tertiary: #201f21;
  --bg-elevated: #2a2a2c;
  --bg-card: #1e1e20;

  --text-primary: #e5e1e4;
  --text-secondary: #cdc2d7;
  --text-muted: #968da0;
  --text-disabled: #6a6a6a;

  --accent-primary: #d6baff;
  --accent-secondary: #adc6ff;
  --accent-tertiary: #ffafd3;

  --border-subtle: rgba(255,255,255,0.06);
  --border-default: rgba(255,255,255,0.10);
  --border-hover: rgba(255,255,255,0.15);

  --sidebar-width: 240px;
  --topbar-height: 56px;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Inter', 'Helvetica Neue', sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  background: var(--bg-primary);
  color: var(--text-primary);
}

*, *::before, *::after {
  box-sizing: border-box;
}

/* ========== 布局框架 ========== */
#app {
  display: flex;
  flex-direction: row;
  background: var(--bg-primary);
}

/* ========== 侧边栏 ========== */
.sidebar {
  width: var(--sidebar-width);
  height: 100vh;
  position: fixed;
  left: 0;
  top: 0;
  background: rgba(19, 19, 21, 0.85);
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  border-right: 1px solid var(--border-default);
  display: flex;
  flex-direction: column;
  padding: 20px 14px;
  z-index: 50;
  transition: width 0.25s ease;
}

.sidebar.collapsed {
  width: 64px;
  padding: 20px 8px;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 28px;
  padding: 0 2px;
}

.sidebar.collapsed .sidebar-header {
  justify-content: center;
  padding: 0;
}

.sidebar-brand {
  font-size: 18px;
  font-weight: 800;
  color: var(--text-primary);
  letter-spacing: -0.02em;
  white-space: nowrap;
  overflow: hidden;
}

.sidebar-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0;
  transition: all 0.2s ease;
}

.sidebar-toggle:hover {
  background: rgba(255,255,255,0.06);
  color: var(--text-primary);
}

.sidebar-toggle .material-icons {
  font-size: 18px;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border-radius: 10px;
  color: var(--text-muted);
  text-decoration: none;
  font-size: 13px;
  font-weight: 500;
  letter-spacing: 0.01em;
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.sidebar.collapsed .nav-item {
  justify-content: center;
  padding: 10px;
}

.nav-item:hover {
  color: var(--text-primary);
  background: rgba(255,255,255,0.04);
}

.nav-item.active {
  color: var(--accent-primary);
  background: rgba(214, 186, 255, 0.10);
  border-color: rgba(214, 186, 255, 0.20);
  font-weight: 600;
}

.nav-icon {
  font-size: 20px;
  width: 24px;
  min-width: 24px;
  text-align: center;
  flex-shrink: 0;
  line-height: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.nav-text {
  font-size: 13px;
  white-space: nowrap;
}

/* ========== 主内容区 ========== */
.main-content {
  margin-left: var(--sidebar-width);
  flex: 1;
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
  position: relative;
  transition: margin-left 0.25s ease;
}

.sidebar.collapsed ~ .main-content {
  margin-left: 64px;
}

/* ========== 顶部栏 ========== */
.topbar {
  height: var(--topbar-height);
  width: 100%;
  background: rgba(19, 19, 21, 0.60);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border-bottom: 1px solid var(--border-default);
  padding: 0 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-shrink: 0;
  z-index: 40;
}

.breadcrumb {
  display: flex;
  align-items: center;
  gap: 8px;
}

.breadcrumb-text {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.version-tag {
  font-size: 11px;
  color: var(--text-muted);
  background: rgba(255,255,255,0.05);
  padding: 3px 10px;
  border-radius: 20px;
  font-weight: 500;
}

/* ========== 页面内容区 ========== */
.page-content {
  flex: 1;
  overflow: hidden;
  position: relative;
}

/* ========== 通用玻璃面板 ========== */
.glass-panel {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid var(--border-default);
  border-top-color: var(--border-hover);
  border-radius: 12px;
  box-shadow: inset 0 1px 0 0 rgba(255,255,255,0.05);
}

.glass-panel-header {
  padding: 14px 18px;
  border-bottom: 1px solid var(--border-default);
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: rgba(255,255,255,0.03);
  border-radius: 12px 12px 0 0;
}

.glass-panel-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.glass-panel-title .material-icons {
  font-size: 18px;
  color: var(--accent-primary);
}

/* ========== 通用按钮 ========== */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid transparent;
  transition: all 0.2s ease;
  background: rgba(255,255,255,0.08);
  color: var(--text-primary);
}

.btn:hover {
  background: rgba(255,255,255,0.12);
}

.btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--accent-primary);
  color: #430089;
  border: none;
  font-weight: 600;
}

.btn-primary:hover {
  opacity: 0.90;
}

.btn-ghost {
  background: transparent;
  border: 1px solid var(--border-default);
  color: var(--text-secondary);
}

.btn-ghost:hover {
  background: rgba(255,255,255,0.05);
  border-color: var(--border-hover);
}

/* ========== 滚动条 ========== */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(255,255,255,0.08);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(255,255,255,0.15);
}

/* ========== 响应式 ========== */
@media (max-width: 768px) {
  .sidebar,
  .sidebar.collapsed {
    width: 60px;
    padding: 16px 8px;
  }

  .sidebar-brand {
    display: none;
  }

  .nav-text {
    display: none;
  }

  .nav-item {
    justify-content: center;
    padding: 10px;
  }

  .main-content,
  .sidebar.collapsed ~ .main-content {
    margin-left: 60px;
  }

  :root {
    --sidebar-width: 60px;
  }
}

/* ========== Material Icons 字体加载 ========== */
@font-face {
  font-family: 'Material Icons';
  font-style: normal;
  font-weight: 400;
  src: url(https://fonts.gstatic.com/s/materialicons/v142/flUhRq6tzZclQEJ-Vdg-IuiaDsNc.woff2) format('woff2');
}

.material-icons {
  font-family: 'Material Icons', sans-serif;
  font-weight: normal;
  font-style: normal;
  font-size: 24px;
  line-height: 1;
  letter-spacing: normal;
  text-transform: none;
  display: inline-block;
  white-space: nowrap;
  word-wrap: normal;
  direction: ltr;
  -webkit-font-feature-settings: 'liga';
  -webkit-font-smoothing: antialiased;
  vertical-align: middle;
}
</style>
