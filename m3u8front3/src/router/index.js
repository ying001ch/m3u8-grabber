import { createRouter, createWebHistory } from 'vue-router'
import Download from '../components/Download.vue'
import Settings from '../components/Settings.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      redirect: '/download'
    },
    {
      path: '/download',
      name: 'download',
      component: Download
    },
    {
      path: '/settings',
      name: 'settings',
      component: Settings
    }
  ]
})

export default router
