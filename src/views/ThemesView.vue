<template>
  <div class="themes-view">
    <header class="page-header">
      <h2>{{ t('themes.title') }}</h2>
      <button 
        class="btn btn-primary"
        @click="handleUpload"
        :disabled="uploadLoading"
      >
        <span v-if="uploadLoading">⏳ {{ t('themes.uploading') }}</span>
        <span v-else>📤 {{ t('themes.upload') }}</span>
      </button>
    </header>
    
    <ThemeSkeleton v-if="loading" :count="6" />
    
    <EmptyState 
      v-else-if="!hasThemes"
      icon="🎨"
      :title="t('themes.empty.title')"
      :description="t('themes.empty.description')"
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
import { onMounted, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'
import { useThemeStore } from '@/stores/theme'
import type { Theme } from '@/types/theme'

const { t } = useI18n()
const themeStore = useThemeStore()

const { themes, currentTheme, loading, uploadLoading, hasThemes } = storeToRefs(themeStore)

onMounted(async () => {
  // 使用 nextTick 确保 DOM 先渲染完成
  await nextTick()
  // 并行加载主题数据，减少等待时间
  await Promise.all([
    themeStore.loadThemes(),
    themeStore.loadCurrentTheme()
  ])
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
</style>
