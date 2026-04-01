import { defineStore } from 'pinia'
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useConfigStore } from './config'

export const useLocaleStore = defineStore('locale', () => {
  const { locale: i18nLocale } = useI18n()
  const configStore = useConfigStore()
  
  // 支持的语言列表
  const availableLocales = [
    { code: 'zh-CN', name: '简体中文' },
    { code: 'en', name: 'English' }
  ]
  
  // 当前语言（从配置 store 同步）
  const currentLocale = computed({
    get: () => configStore.config.locale || 'zh-CN',
    set: (value: string) => {
      configStore.config.locale = value
    }
  })
  
  // 计算属性：当前语言的显示名称
  const currentLocaleName = computed(() => {
    const lang = availableLocales.find(l => l.code === currentLocale.value)
    return lang?.name || '简体中文'
  })
  
  // 初始化语言设置
  function initLocale() {
    // 语言设置已从后端配置加载到 configStore
    // 直接应用即可
    const targetLocale = currentLocale.value
    
    // 同步设置语言
    i18nLocale.value = targetLocale
    
    // 异步通知后端（不阻塞 UI）
    setTimeout(() => {
      notifyBackend(targetLocale)
    }, 0)
  }
  
  // 设置语言
  function setLocale(code: string) {
    if (!availableLocales.some(l => l.code === code)) {
      console.warn(`Unsupported locale: ${code}`)
      return
    }
    
    // 同步更新
    currentLocale.value = code
    i18nLocale.value = code
    
    // 保存配置到后端
    setTimeout(() => {
      configStore.saveConfig()
    }, 0)
    
    // 通知后端（托盘菜单将在下次启动时更新）
    setTimeout(() => {
      notifyBackend(code)
    }, 0)
  }
  
  // 通知后端更新托盘菜单
  async function notifyBackend(code: string) {
    try {
      if (typeof window !== 'undefined' && window.__TAURI__) {
        const { invoke } = window.__TAURI__.core
        await invoke('set_locale', { locale: code })
      }
    } catch (e) {
      console.error('Failed to notify backend of locale change:', e)
    }
  }
  
  return {
    availableLocales,
    currentLocale,
    currentLocaleName,
    initLocale,
    setLocale
  }
})
