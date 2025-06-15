import { createRouter, createWebHistory, Router } from 'vue-router'

const routes = [
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
  },
  {
    path: '/:catchAll(.*)',
    redirect: '/json-parser'
  }
]

const router: Router = createRouter({
  history: createWebHistory(),
  routes
})

export default router 