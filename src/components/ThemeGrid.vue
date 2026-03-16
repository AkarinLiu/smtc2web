<template>
  <div class="themes-grid">
    <ThemeCard
      v-for="theme in themes"
      :key="theme.folder_name"
      :theme="theme"
      :is-active="theme.folder_name === currentTheme"
      :screenshot-url="getScreenshotUrl(theme.screenshot_path)"
      @select="$emit('select', theme.folder_name)"
      @delete="$emit('delete', theme)"
    />
  </div>
</template>

<script setup lang="ts">
import type { Theme } from '@/types/theme'

interface Props {
  themes: Theme[]
  currentTheme: string
}

defineProps<Props>()
defineEmits<{
  select: [folderName: string]
  delete: [theme: Theme]
}>()

function getScreenshotUrl(path: string): string | null {
  return path && path.startsWith('data:') ? path : null
}
</script>

<style scoped>
.themes-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--fluent-space-lg);
}

@media (max-width: 768px) {
  .themes-grid {
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: var(--fluent-space-md);
  }
}

@media (max-width: 480px) {
  .themes-grid {
    grid-template-columns: 1fr;
  }
}
</style>
