import { createRouter, createWebHistory } from 'vue-router'

// 懒加载路由组件
const ThemesView = () => import('@/views/ThemesView.vue')
const SettingsView = () => import('@/views/SettingsView.vue')

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/themes'
    },
    {
      path: '/themes',
      name: 'themes',
      component: ThemesView
    },
    {
      path: '/settings',
      name: 'settings',
      component: SettingsView
    }
  ]
})

export default router
