import { createApp } from "vue";
import App from "./App.vue";
import { createPinia } from 'pinia';
import router from './router';

const app = createApp(App);

// 使用 Pinia 状态管理
const pinia = createPinia();
app.use(pinia);

// 使用路由
app.use(router);

// 全局错误处理
app.config.errorHandler = (err: any, _: any, info: any) => {
  console.error('全局错误:', err);
  console.error('错误信息:', info);
};

app.mount("#app");
