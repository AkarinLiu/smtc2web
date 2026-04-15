import { defineStore } from 'pinia'
import { reactive, ref, computed } from 'vue'
import type { AppConfig } from '@/types/config'

export const useConfigStore = defineStore('config', () => {
  const config = reactive<AppConfig>({
    server_port: 3030,
    address: '127.0.0.1',
    show_console: false,
    current_theme: '',
    locale: 'zh-CN',
    process_filter: '*'
  })

  const loading = ref(false)
  const saved = ref(false)
  const currentAppId = ref('')

  const hasTauri = computed(() => {
    return typeof window !== 'undefined' &&
           window.__TAURI__ !== undefined
  })

  async function loadConfig() {
    try {
      if (hasTauri.value && window.__TAURI__) {
        const { invoke } = window.__TAURI__.core
        const data = await invoke<AppConfig>('get_config')
        Object.assign(config, data)
      }
    } catch (e) {
      console.error('加载配置失败:', e)
    }
  }

  async function saveConfig() {
    loading.value = true
    saved.value = false

    try {
      if (hasTauri.value && window.__TAURI__) {
        const { invoke } = window.__TAURI__.core
        await invoke('save_config', { configDto: config })
      }
      saved.value = true
      setTimeout(() => saved.value = false, 2000)
    } catch (e) {
      console.error('保存配置失败:', e)
      alert('保存配置失败: ' + e)
    } finally {
      loading.value = false
    }
  }

  async function getCurrentAppId() {
    try {
      if (hasTauri.value && window.__TAURI__) {
        const { invoke } = window.__TAURI__.core
        const appId = await invoke<string>('get_current_app_id')
        currentAppId.value = appId
        return appId
      }
    } catch (e) {
      console.error('获取当前应用ID失败:', e)
    }
    return ''
  }

  return {
    config,
    loading,
    saved,
    currentAppId,
    loadConfig,
    saveConfig,
    getCurrentAppId
  }
})
