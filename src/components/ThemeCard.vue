<template>
  <div class="theme-card" :class="{ active: isActive }" @click="$emit('select')">
    <div class="theme-preview">
      <img v-if="screenshotUrl" :src="screenshotUrl" :alt="theme.name" />
      <div v-else class="preview-placeholder">
        <span class="placeholder-icon"><font-awesome-icon icon="image" /></span>
        <span>{{ t('themes.card.noPreview') }}</span>
      </div>
      <div v-if="isActive" class="active-badge">
        <span class="badge-dot"></span>
        <span>{{ t('themes.card.active') }}</span>
      </div>
      <div v-if="theme.is_default" class="default-badge">
        <span>{{ t('themes.card.default') }}</span>
      </div>
      <div v-if="isGit" class="git-badge">
        <font-awesome-icon icon="code-branch" />
        <span>{{ t('themes.card.gitBadge') }}</span>
      </div>
    </div>
    <div class="theme-info">
      <h3 class="theme-name">{{ theme.name }}</h3>
      <div class="theme-meta">
        <span class="theme-author">{{ theme.author }}</span>
        <span class="theme-version">{{ theme.version }}</span>
      </div>
    </div>
    <div class="card-actions">
      <button
        v-if="isGit && hasUpdate"
        class="update-btn"
        @click.stop="handleUpdate"
        :title="t('themes.card.update')"
        :disabled="updating"
      >
        <font-awesome-icon v-if="updating" icon="spinner" spin />
        <font-awesome-icon v-else icon="rotate" />
      </button>
      <button
        v-if="!isActive && !theme.is_default"
        class="delete-btn"
        @click.stop="$emit('delete')"
        :title="t('themes.card.delete')"
      >
        <font-awesome-icon icon="trash" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useThemeStore } from '@/stores/theme'
import type { Theme } from '@/types/theme'

interface Props {
  theme: Theme
  isActive: boolean
  screenshotUrl: string | null
}

const { t } = useI18n()
const themeStore = useThemeStore()

const props = defineProps<Props>()
defineEmits<{
  select: []
  delete: []
}>()

const isGit = computed(() => themeStore.isGitTheme(props.theme.folder_name))
const hasUpdate = ref(false)
const updating = ref(false)

onMounted(async () => {
  if (isGit.value) {
    hasUpdate.value = await themeStore.checkGitUpdate(props.theme.folder_name)
  }
})

async function handleUpdate() {
  if (updating.value) return
  updating.value = true
  try {
    await themeStore.updateGitTheme(props.theme.folder_name)
    hasUpdate.value = false
  } finally {
    updating.value = false
  }
}
</script>

<style scoped>
.theme-card {
  background-color: var(--ui-bg-card);
  border-radius: var(--ui-radius-lg);
  overflow: hidden;
  box-shadow: var(--ui-shadow-md);
  transition: all var(--ui-transition-normal);
  cursor: pointer;
  position: relative;
  border: 2px solid transparent;
}

.theme-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--ui-shadow-lg);
}

.theme-card.active {
  border-color: var(--ui-accent);
}

.theme-preview {
  position: relative;
  width: 100%;
  aspect-ratio: 16 / 9;
  background: linear-gradient(135deg, var(--ui-bg-secondary) 0%, var(--ui-bg-tertiary) 100%);
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
  color: var(--ui-text-secondary);
  font-size: 14px;
  gap: var(--ui-space-sm);
}

.placeholder-icon {
  font-size: 32px;
  opacity: 0.5;
}

.active-badge {
  position: absolute;
  bottom: var(--ui-space-sm);
  left: var(--ui-space-sm);
  background-color: var(--ui-success);
  color: var(--ui-text-on-accent);
  padding: var(--ui-space-xs) var(--ui-space-sm);
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
  top: var(--ui-space-sm);
  left: var(--ui-space-sm);
  background-color: var(--ui-accent);
  color: var(--ui-text-on-accent);
  padding: var(--ui-space-xs) var(--ui-space-sm);
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.git-badge {
  position: absolute;
  top: var(--ui-space-sm);
  right: var(--ui-space-sm);
  background-color: var(--ui-accent);
  color: var(--ui-text-on-accent);
  padding: var(--ui-space-xs) var(--ui-space-sm);
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 4px;
}

.card-actions {
  position: absolute;
  top: var(--ui-space-sm);
  left: var(--ui-space-sm);
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity var(--ui-transition-fast);
}

.theme-card:hover .card-actions {
  opacity: 1;
}

.update-btn {
  width: 32px;
  height: 32px;
  background-color: var(--ui-success);
  color: white;
  border: none;
  border-radius: 50%;
  font-size: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform var(--ui-transition-fast);
}

.update-btn:hover:not(:disabled) {
  transform: scale(1.1);
}

.update-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.theme-info {
  padding: var(--ui-space-md);
}

.theme-name {
  font-size: 16px;
  font-weight: 600;
  margin-bottom: var(--ui-space-xs);
  color: var(--ui-text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.theme-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
  color: var(--ui-text-secondary);
}

.theme-author {
  max-width: 60%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.theme-version {
  background-color: var(--ui-bg-secondary);
  padding: 2px 8px;
  border-radius: var(--ui-radius-sm);
  font-size: 12px;
  font-weight: 500;
}

.delete-btn {
  width: 32px;
  height: 32px;
  background-color: var(--ui-error);
  color: white;
  border: none;
  border-radius: 50%;
  font-size: 16px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform var(--ui-transition-fast);
}

.delete-btn:hover {
  transform: scale(1.1);
}
</style>
