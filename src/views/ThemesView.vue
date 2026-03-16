<template>
  <div class="themes-view">
    <header class="page-header">
      <h2>主题列表</h2>
      <button 
        class="btn btn-primary"
        @click="handleUpload"
        :disabled="uploadLoading"
      >
        <span v-if="uploadLoading">⏳ 上传中...</span>
        <span v-else>📤 上传主题</span>
      </button>
    </header>
    
    <div v-if="loading" class="loading">
      <span class="spinner">⏳</span>
      <span>加载中...</span>
    </div>
    
    <EmptyState 
      v-else-if="!hasThemes"
      icon="🎨"
      title="暂无主题"
      description="点击「上传主题」按钮添加新主题"
    />
    
    <ThemeGrid
      v-else
      :themes="themes"
      :current-theme="currentTheme"
      @select="handleSelect"
      @delete="handleDelete"
    />
  </div>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { onMounted } from 'vue'
import { useThemeStore } from '@/stores/theme'
import type { Theme } from '@/types/theme'

const themeStore = useThemeStore()

const { themes, currentTheme, loading, uploadLoading, hasThemes } = storeToRefs(themeStore)

onMounted(() => {
  themeStore.loadThemes()
  themeStore.loadCurrentTheme()
})

function handleSelect(folderName: string) {
  themeStore.selectTheme(folderName)
}

function handleDelete(theme: Theme) {
  themeStore.deleteTheme(theme)
}

function handleUpload() {
  themeStore.uploadTheme()
}
</script>

<style scoped>
.themes-view {
  width: 100%;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: var(--fluent-radius-md);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--fluent-transition-fast);
  display: inline-flex;
  align-items: center;
  gap: var(--fluent-space-xs);
}

.btn-primary {
  background-color: var(--fluent-accent);
  color: var(--fluent-text-on-accent);
}

.btn-primary:hover:not(:disabled) {
  background-color: var(--fluent-accent-hover);
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px;
  gap: var(--fluent-space-md);
  color: var(--fluent-text-secondary);
  font-size: 16px;
}

.spinner {
  font-size: 32px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
