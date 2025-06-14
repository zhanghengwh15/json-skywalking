import { createRouter, createWebHistory, Router } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: () => import('../views/Home.vue')
  },
  {
    path: '/json-parser',
    name: 'JsonParser',
    component: () => import('../views/JsonParser.vue')
  },
  {
    path: '/sql-parser',
    name: 'SqlParser',
    component: () => import('../views/SqlParser.vue')
  },
  {
    path: '/post-parser',
    name: 'PostParser',
    component: () => import('../views/PostParser.vue')
  }
]

const router: Router = createRouter({
  history: createWebHistory(),
  routes
})

export default router 