<template>
  <div class="home">
    <div class="hero-section">
      <h1 class="main-title">开发工具集合</h1>
      <p class="subtitle">高效的开发辅助工具，提升您的开发效率</p>
    </div>

    <div class="features-grid">
      <!-- JSON 解析工具 -->
      <div class="feature-card json-card">
        <div class="feature-icon">🔧</div>
        <h3 class="feature-title">JSON 解析工具</h3>
        <p class="feature-description">
          智能JSON数据解析与格式化工具，支持多种输入格式和美化显示
        </p>
        <div class="feature-highlights">
          <div class="highlight-item">✨ 智能格式识别</div>
          <div class="highlight-item">🎨 语法高亮显示</div>
          <div class="highlight-item">🔄 多格式转换</div>
          <div class="highlight-item">📋 一键复制结果</div>
        </div>
        <button @click="navigateToJsonParser" class="nav-button json-button">
          开始使用 JSON 工具
        </button>
      </div>

      <!-- HTTP 解析工具 -->
      <div class="feature-card http-card">
        <div class="feature-icon">🌐</div>
        <h3 class="feature-title">HTTP 请求解析</h3>
        <p class="feature-description">
          专业的HTTP请求解析工具，支持GET/POST请求，自动生成cURL命令
        </p>
        <div class="feature-highlights">
          <div class="highlight-item">🚀 GET/POST 请求解析</div>
          <div class="highlight-item">⚡ 自动生成 cURL 命令</div>
          <div class="highlight-item">📚 智能历史记录</div>
          <div class="highlight-item">🔍 快速搜索过滤</div>
        </div>
        <button @click="navigateToHttpParser" class="nav-button http-button">
          开始使用 HTTP 工具
        </button>
      </div>
    </div>

    <!-- 快速访问区域 -->
    <div class="quick-access">
      <h2 class="section-title">快速访问</h2>
      <div class="quick-nav-grid">
        <div class="quick-nav-item" @click="navigateToJsonParser">
          <div class="quick-icon">🔧</div>
          <span>JSON 解析</span>
        </div>
        <div class="quick-nav-item" @click="navigateToHttpParser">
          <div class="quick-icon">🌐</div>
          <span>HTTP 解析</span>
        </div>
        <div class="quick-nav-item" @click="navigateToSqlParser">
          <div class="quick-icon">🗃️</div>
          <span>SQL 解析</span>
        </div>
      </div>
    </div>

    <!-- 使用技巧 -->
    <div class="tips-section">
      <h2 class="section-title">使用技巧</h2>
      <div class="tips-grid">
        <div class="tip-card">
          <div class="tip-icon">💡</div>
          <h4>JSON 工具使用</h4>
          <p>支持从剪贴板直接粘贴数据，自动识别格式并美化显示</p>
        </div>
        <div class="tip-card">
          <div class="tip-icon">⚡</div>
          <h4>HTTP 历史记录</h4>
          <p>所有解析的请求会自动保存，支持搜索和批量管理</p>
        </div>
        <div class="tip-card">
          <div class="tip-icon">🎯</div>
          <h4>快捷操作</h4>
          <p>使用键盘快捷键和右键菜单快速完成常用操作</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

// 导航函数
const navigateToJsonParser = () => {
  router.push('/json-parser')
}

const navigateToHttpParser = () => {
  router.push('/post-parser')
}

const navigateToSqlParser = () => {
  router.push('/sql-parser')
}
</script>

<style scoped>
.home {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
  min-height: calc(100vh - 80px);
}

/* 头部英雄区域 */
.hero-section {
  text-align: center;
  padding: 40px 0 60px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  margin: -20px -20px 40px -20px;
  color: white;
  border-radius: 0 0 20px 20px;
}

.main-title {
  font-size: 2.5rem;
  font-weight: 700;
  margin-bottom: 15px;
  text-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.subtitle {
  font-size: 1.2rem;
  opacity: 0.9;
  margin: 0;
  font-weight: 300;
}

/* 功能卡片网格 */
.features-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
  gap: 30px;
  margin-bottom: 50px;
}

.feature-card {
  background: white;
  border-radius: 16px;
  padding: 30px;
  box-shadow: 0 8px 30px rgba(0,0,0,0.1);
  transition: all 0.3s ease;
  border: 1px solid #f0f0f0;
}

.feature-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 15px 40px rgba(0,0,0,0.15);
}

.json-card {
  border-left: 4px solid #3498db;
}

.http-card {
  border-left: 4px solid #e74c3c;
}

.feature-icon {
  font-size: 3rem;
  margin-bottom: 20px;
  display: block;
}

.feature-title {
  font-size: 1.5rem;
  font-weight: 600;
  margin-bottom: 15px;
  color: #2c3e50;
}

.feature-description {
  color: #666;
  line-height: 1.6;
  margin-bottom: 20px;
  font-size: 1rem;
}

.feature-highlights {
  margin-bottom: 25px;
}

.highlight-item {
  padding: 8px 0;
  color: #555;
  font-size: 0.95rem;
  display: flex;
  align-items: center;
}

.highlight-item::before {
  content: '';
  width: 4px;
  height: 4px;
  background: #3498db;
  border-radius: 50%;
  margin-right: 12px;
}

.http-card .highlight-item::before {
  background: #e74c3c;
}

/* 导航按钮 */
.nav-button {
  width: 100%;
  padding: 12px 24px;
  border: none;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  color: white;
}

.json-button {
  background: linear-gradient(135deg, #3498db, #2980b9);
}

.json-button:hover {
  background: linear-gradient(135deg, #2980b9, #1f618d);
  transform: translateY(-2px);
}

.http-button {
  background: linear-gradient(135deg, #e74c3c, #c0392b);
}

.http-button:hover {
  background: linear-gradient(135deg, #c0392b, #a93226);
  transform: translateY(-2px);
}

/* 快速访问区域 */
.quick-access {
  margin-bottom: 50px;
}

.section-title {
  font-size: 1.8rem;
  font-weight: 600;
  color: #2c3e50;
  margin-bottom: 25px;
  text-align: center;
}

.quick-nav-grid {
  display: flex;
  justify-content: center;
  gap: 20px;
  flex-wrap: wrap;
}

.quick-nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 25px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 4px 15px rgba(0,0,0,0.1);
  cursor: pointer;
  transition: all 0.3s ease;
  border: 2px solid transparent;
  min-width: 200px;
}

.quick-nav-item:hover {
  transform: translateY(-3px);
  box-shadow: 0 8px 25px rgba(0,0,0,0.15);
  border-color: #3498db;
}

.quick-icon {
  font-size: 2rem;
  margin-bottom: 10px;
}

.quick-nav-item span {
  font-weight: 500;
  color: #2c3e50;
}

/* 使用技巧区域 */
.tips-section {
  background: #f8f9fa;
  padding: 40px;
  border-radius: 16px;
  margin: -20px;
}

.tips-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 25px;
}

.tip-card {
  background: white;
  padding: 25px;
  border-radius: 12px;
  box-shadow: 0 2px 10px rgba(0,0,0,0.05);
  transition: all 0.3s ease;
}

.tip-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 20px rgba(0,0,0,0.1);
}

.tip-icon {
  font-size: 1.8rem;
  margin-bottom: 15px;
}

.tip-card h4 {
  font-size: 1.1rem;
  font-weight: 600;
  color: #2c3e50;
  margin-bottom: 10px;
}

.tip-card p {
  color: #666;
  line-height: 1.5;
  margin: 0;
  font-size: 0.95rem;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .home {
    padding: 10px;
  }
  
  .hero-section {
    padding: 30px 20px 40px;
    margin: -10px -10px 30px -10px;
  }
  
  .main-title {
    font-size: 2rem;
  }
  
  .subtitle {
    font-size: 1rem;
  }
  
  .features-grid {
    grid-template-columns: 1fr;
    gap: 20px;
  }
  
  .feature-card {
    padding: 20px;
  }
  
  .quick-nav-grid {
    justify-content: center;
    gap: 15px;
  }
  
  .quick-nav-item {
    padding: 20px 15px;
  }
  
  .tips-section {
    padding: 30px 20px;
    margin: -10px;
  }
  
  .tips-grid {
    grid-template-columns: 1fr;
    gap: 20px;
  }
}

@media (max-width: 480px) {
  .quick-nav-grid {
    flex-direction: column;
    align-items: center;
  }
  
  .features-grid {
    grid-template-columns: 1fr;
  }
}
</style> 