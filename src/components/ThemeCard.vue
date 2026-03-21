<template>
  <div class="theme-card" :class="{ active: isActive }" @click="$emit('select')">
    <div class="theme-preview">
      <img v-if="screenshotUrl" :src="screenshotUrl" :alt="theme.name" />
      <div v-else class="preview-placeholder">
        <span class="placeholder-icon">🖼️</span>
        <span>{{ t('themes.card.noPreview') }}</span>
      </div>
      <div v-if="isActive" class="active-badge">
        <span class="badge-dot"></span>
        <span>{{ t('themes.card.active') }}</span>
      </div>
      <div v-if="theme.is_default" class="default-badge">
        <span>{{ t('themes.card.default') }}</span>
      </div>
    </div>
    <div class="theme-info">
      <h3 class="theme-name">{{ theme.name }}</h3>
      <div class="theme-meta">
        <span class="theme-author">{{ theme.author }}</span>
        <span class="theme-version">{{ theme.version }}</span>
      </div>
    </div>
    <button 
      v-if="!isActive && !theme.is_default"
      class="delete-btn" 
      @click.stop="$emit('delete')"
      :title="t('themes.card.delete')"
    >
      🗑️
    </button>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import type { Theme } from '@/types/theme'

interface Props {
  theme: Theme
  isActive: boolean
  screenshotUrl: string | null
}

const { t } = useI18n()

defineProps<Props>()
defineEmits<{
  select: []
  delete: []
}>()
</script>

<style scoped>
.theme-card {
  background-color: var(--fluent-bg-card);
  border-radius: var(--fluent-radius-lg);
  overflow: hidden;
  box-shadow: var(--fluent-shadow-md);
  transition: all var(--fluent-transition-normal);
  cursor: pointer;
  position: relative;
  border: 2px solid transparent;
}

.theme-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--fluent-shadow-lg);
}

.theme-card.active {
  border-color: var(--fluent-accent);
}

.theme-preview {
  position: relative;
  width: 100%;
  aspect-ratio: 16 / 9;
  background: linear-gradient(135deg, var(--fluent-bg-secondary) 0%, var(--fluent-bg-tertiary) 100%);
  overflow: hidden;
}

.theme-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.preview-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--fluent-text-secondary);
  font-size: 14px;
  gap: var(--fluent-space-sm);
}

.placeholder-icon {
  font-size: 32px;
  opacity: 0.5;
}

.active-badge {
  position: absolute;
  bottom: var(--fluent-space-sm);
  left: var(--fluent-space-sm);
  background-color: var(--fluent-success);
  color: var(--fluent-text-on-accent);
  padding: var(--fluent-space-xs) var(--fluent-space-sm);
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 4px;
}

.badge-dot {
  width: 6px;
  height: 6px;
  background-color: white;
  border-radius: 50%;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.default-badge {
  position: absolute;
  top: var(--fluent-space-sm);
  left: var(--fluent-space-sm);
  background-color: var(--fluent-accent);
  color: var(--fluent-text-on-accent);
  padding: var(--fluent-space-xs) var(--fluent-space-sm);
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.theme-info {
  padding: var(--fluent-space-md);
}

.theme-name {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: var(--fluent-space-xs);
  color: var(--fluent-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.theme-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
  color: var(--fluent-text-secondary);
}

.theme-author {
  max-width: 60%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.theme-version {
  background-color: var(--fluent-bg-secondary);
  padding: 2px 8px;
  border-radius: var(--fluent-radius-sm);
  font-size: 12px;
  font-weight: 500;
}

.delete-btn {
  position: absolute;
  top: var(--fluent-space-sm);
  right: var(--fluent-space-sm);
  width: 32px;
  height: 32px;
  background-color: var(--fluent-error);
  color: white;
  border: none;
  border-radius: 50%;
  font-size: 16px;
  cursor: pointer;
  opacity: 0;
  transition: opacity var(--fluent-transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
}

.theme-card:hover .delete-btn {
  opacity: 1;
}

.delete-btn:hover {
  transform: scale(1.1);
}
</style>
