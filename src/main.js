// 使用全局 Vue（通过 Tauri 的 withGlobalTauri）
const { createApp, ref, reactive, computed } = Vue
const { invoke, convertFileSrc } = window.__TAURI__.core
const { open } = window.__TAURI__.dialog

// 创建应用
createApp({
  setup() {
    const currentView = ref('themes')
    const currentTheme = ref('')
    const themes = ref([])
    const themesLoading = ref(false)
    const config = reactive({
      server_port: 3030,
      address: '127.0.0.1',
      show_console: false,
      current_theme: ''
    })
    const configLoading = ref(false)
    const configSaved = ref(false)
    const uploadLoading = ref(false)

    // 加载主题列表
    const loadThemes = async () => {
      themesLoading.value = true
      try {
        themes.value = await invoke('get_themes')
      } catch (e) {
        console.error('加载主题失败:', e)
      } finally {
        themesLoading.value = false
      }
    }

    // 加载当前主题
    const loadCurrentTheme = async () => {
      try {
        currentTheme.value = await invoke('get_current_theme')
      } catch (e) {
        console.error('加载当前主题失败:', e)
      }
    }

    // 切换主题
    const selectTheme = async (theme) => {
      if (theme.folder_name === currentTheme.value) return
      
      try {
        await invoke('set_theme', { themeName: theme.folder_name })
        await loadCurrentTheme()
        alert('主题切换成功！')
      } catch (e) {
        console.error('切换主题失败:', e)
        alert('切换主题失败: ' + e)
      }
    }

    // 删除主题
    const deleteTheme = async (theme) => {
      if (theme.is_default) {
        alert('默认主题不能删除')
        return
      }
      
      if (!confirm(`确定要删除主题 "${theme.name}" 吗？`)) return
      
      try {
        await invoke('delete_theme', { themeFolder: theme.folder_name })
        await loadThemes()
      } catch (e) {
        console.error('删除主题失败:', e)
        alert('删除主题失败: ' + e)
      }
    }

    // 上传主题
    const uploadTheme = async () => {
      try {
        const selected = await open({
          multiple: false,
          filters: [{
            name: '压缩文件',
            extensions: ['zip']
          }]
        })

        if (!selected) return

        const filePath = Array.isArray(selected) ? selected[0] : selected
        
        uploadLoading.value = true
        await invoke('upload_theme', { filePath })
        await loadThemes()
        alert('主题上传成功！')
      } catch (e) {
        console.error('上传主题失败:', e)
        alert('上传主题失败: ' + e)
      } finally {
        uploadLoading.value = false
      }
    }

    // 获取截图 URL
    const getScreenshotUrl = (path) => {
      if (!path) return null
      return convertFileSrc(path)
    }

    // 加载配置
    const loadConfig = async () => {
      try {
        const data = await invoke('get_config')
        Object.assign(config, data)
      } catch (e) {
        console.error('加载配置失败:', e)
      }
    }

    // 保存配置
    const saveConfig = async () => {
      configLoading.value = true
      configSaved.value = false
      
      try {
        await invoke('save_config', { configDto: config })
        configSaved.value = true
        setTimeout(() => configSaved.value = false, 2000)
      } catch (e) {
        console.error('保存配置失败:', e)
        alert('保存配置失败: ' + e)
      } finally {
        configLoading.value = false
      }
    }

    // 初始加载
    loadThemes()
    loadCurrentTheme()
    loadConfig()

    return {
      currentView,
      currentTheme,
      themes,
      themesLoading,
      config,
      configLoading,
      configSaved,
      uploadLoading,
      selectTheme,
      deleteTheme,
      uploadTheme,
      getScreenshotUrl,
      saveConfig
    }
  }
}).mount('#app')
