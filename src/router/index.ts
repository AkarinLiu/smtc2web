import { createRouter, createWebHistory } from 'vue-router'
import ThemesView from '@/views/ThemesView.vue'
import SettingsView from '@/views/SettingsView.vue'

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
