import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Theme } from '@/types/theme'

export const useThemeStore = defineStore('theme', () => {
  const themes = ref<Theme[]>([])
  const currentTheme = ref('')
  const loading = ref(false)
  const uploadLoading = ref(false)
  
  const hasThemes = computed(() => themes.value.length > 0)
  
  const hasTauri = computed(() => {
    return typeof window !== 'undefined' && 
           window.__TAURI__ !== undefined
  })
  
  const mockThemes: Theme[] = [
    {
      name: '默认主题',
      folder_name: 'default',
      author: 'smtc2web',
      version: '1.0.0',
      screenshot_path: '',
      is_default: true
    },
    {
      name: '深色主题',
      folder_name: 'dark',
      author: '用户',
      version: '2.0.0',
      screenshot_path: ''
    }
  ]
  
  async function loadThemes() {
    loading.value = true
    try {
      if (hasTauri.value && window.__TAURI__) {
        const { invoke } = window.__TAURI__.core
        themes.value = await invoke<Theme[]>('get_themes')
      } else {
        themes.value = mockThemes
      }
    } catch (e) {
      console.error('加载主题失败:', e)
      themes.value = mockThemes
    } finally {
      loading.value = false
    }
  }
  
  async function loadCurrentTheme() {
    try {
      if (hasTauri.value && window.__TAURI__) {
        const { invoke } = window.__TAURI__.core
        currentTheme.value = await invoke<string>('get_current_theme')
      } else {
        currentTheme.value = 'default'
      }
    } catch (e) {
      console.error('加载当前主题失败:', e)
    }
  }
  
  async function selectTheme(folderName: string) {
    if (folderName === currentTheme.value) return
    
    try {
      if (hasTauri.value && window.__TAURI__) {
        const { invoke } = window.__TAURI__.core
        await invoke('set_theme', { themeName: folderName })
        await loadCurrentTheme()
        alert('主题切换成功！')
      } else {
        currentTheme.value = folderName
        alert('主题切换成功！(模拟模式)')
      }
    } catch (e) {
      console.error('切换主题失败:', e)
      alert('切换主题失败: ' + e)
    }
  }
  
  async function deleteTheme(theme: Theme) {
    if (!confirm(`确定要删除主题 "${theme.name}" 吗？`)) return
    
    try {
      if (hasTauri.value && window.__TAURI__) {
        const { invoke } = window.__TAURI__.core
        await invoke('delete_theme', { themeFolder: theme.folder_name })
        await loadThemes()
      } else {
        themes.value = themes.value.filter((t: Theme) => t.folder_name !== theme.folder_name)
      }
    } catch (e) {
      console.error('删除主题失败:', e)
      alert('删除主题失败: ' + e)
    }
  }
  
  async function uploadTheme() {
    if (!hasTauri.value) {
      alert('上传功能需要 Tauri 环境')
      return
    }
    
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = '.zip'
    
    input.onchange = async (e: Event) => {
      const file = (e.target as HTMLInputElement).files?.[0]
      if (!file) return
      
      if (!file.name.endsWith('.zip')) {
        alert('请选择 ZIP 格式的主题文件')
        return
      }
      
      uploadLoading.value = true
      try {
        const arrayBuffer = await file.arrayBuffer()
        const uint8Array = new Uint8Array(arrayBuffer)
        
        if (window.__TAURI__) {
          const { invoke } = window.__TAURI__.core
          const themeName = await invoke<string>('upload_theme_from_bytes', {
            fileName: file.name,
            fileData: Array.from(uint8Array)
          })
          
          await new Promise(resolve => setTimeout(resolve, 500))
          await loadThemes()
          alert(`主题 "${themeName}" 上传成功！`)
        }
      } catch (e: any) {
        console.error('上传主题失败:', e)
        alert('上传主题失败: ' + (e.message || e))
      } finally {
        uploadLoading.value = false
      }
    }
    
    input.click()
  }
  
  function getScreenshotUrl(path: string): string | null {
    return path && path.startsWith('data:') ? path : null
  }
  
  return {
    themes,
    currentTheme,
    loading,
    uploadLoading,
    hasThemes,
    loadThemes,
    loadCurrentTheme,
    selectTheme,
    deleteTheme,
    uploadTheme,
    getScreenshotUrl
  }
})
