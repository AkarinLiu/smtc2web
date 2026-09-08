<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay class="dialog-overlay" @click="closeDialog" />
      <DialogContent class="dialog-content">
        <div class="dialog-header">
          <font-awesome-icon icon="circle-up" class="dialog-icon" />
          <DialogTitle as="h3">{{ t('update.title') }}</DialogTitle>
        </div>

        <div class="dialog-body">
          <!-- 检查错误状态 -->
          <div v-if="lastResult?.error" class="dialog-error">
            <p>{{ lastResult.error }}</p>
          </div>

          <!-- 下载错误状态 -->
          <div v-else-if="updateStore.downloadError" class="dialog-error">
            <p>{{ t('update.downloadError') }}: {{ updateStore.downloadError }}</p>
          </div>

          <!-- 无更新 -->
          <div v-else-if="lastResult && !lastResult.has_update" class="dialog-no-update">
            <p>
              <font-awesome-icon icon="circle-check" />
              {{ t('update.noUpdate') }}
            </p>
          </div>

          <!-- 有更新 -->
          <div v-else-if="lastResult?.has_update" class="dialog-has-update">
            <div class="version-info">
              <div class="version-row">
                <span class="label">{{ t('update.currentVersion') }}</span>
                <code>{{ lastResult.current_version }}</code>
              </div>
              <div class="version-row">
                <span class="label">{{ t('update.latestVersion') }}</span>
                <code class="latest-ver">{{ lastResult.latest_version }}</code>
              </div>
            </div>

            <div v-if="lastResult.notes" class="release-notes">
              <h4>{{ t('update.releaseNotes') }}</h4>
              <p>{{ lastResult.notes }}</p>
            </div>
          </div>
        </div>

        <div class="dialog-footer">
          <!-- 有更新时显示下载按钮 -->
          <button
            v-if="lastResult?.has_update && !updateStore.downloadError"
            class="btn btn-primary"
            @click="handleDownload"
            :disabled="updateStore.downloading"
          >
            <font-awesome-icon icon="download" :spin="updateStore.downloading" />
            {{ updateStore.downloading ? t('update.downloading') : t('update.download') }}
          </button>

          <DialogClose as-child>
            <button class="btn btn-secondary">
              {{ lastResult?.has_update ? t('update.later') : t('common.confirm') }}
            </button>
          </DialogClose>
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  DialogClose,
  DialogContent,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from 'reka-ui'
import { useUpdateStore } from '@/stores/update'

const { t } = useI18n()
const updateStore = useUpdateStore()

const lastResult = computed(() => updateStore.lastResult)

const open = computed({
  get: () => updateStore.showDialog,
  set: (v: boolean) => {
    if (!v) closeDialog()
  },
})

function closeDialog() {
  updateStore.closeDialog()
}

async function handleDownload() {
  try {
    await updateStore.downloadAndInstall()
  } catch (e) {
    console.error('下载更新失败:', e)
  }
}
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  z-index: 9998;
}

.dialog-overlay[data-state='open'] {
  animation: dialog-fade-in 0.2s ease;
}

.dialog-overlay[data-state='closed'] {
  animation: dialog-fade-out 0.2s ease;
}

.dialog-content {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 9999;
  background: var(--ui-bg-card);
  border-radius: var(--ui-radius-lg);
  box-shadow: var(--ui-shadow-lg);
  max-width: 440px;
  width: 90%;
  padding: var(--ui-space-lg);
  outline: none;
}

.dialog-content[data-state='open'] {
  animation: dialog-fade-in 0.2s ease;
}

.dialog-content[data-state='closed'] {
  animation: dialog-fade-out 0.2s ease;
}

.dialog-header {
  display: flex;
  align-items: center;
  gap: var(--ui-space-sm);
  margin-bottom: var(--ui-space-lg);
}

.dialog-icon {
  font-size: 24px;
  color: var(--ui-accent);
}

.dialog-header h3 {
  margin: 0;
  font-size: 18px;
  color: var(--ui-text-primary);
}

.dialog-body {
  margin-bottom: var(--ui-space-lg);
}

.dialog-error {
  color: var(--ui-error);
  font-size: 14px;
}

.dialog-no-update {
  color: var(--ui-success);
  font-size: 14px;
}

.version-info {
  background: var(--ui-bg-secondary);
  border-radius: var(--ui-radius-md);
  padding: var(--ui-space-md);
  margin-bottom: var(--ui-space-md);
}

.version-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 0;
}

.version-row .label {
  font-size: 13px;
  color: var(--ui-text-secondary);
}

.version-row code {
  font-size: 14px;
  font-weight: 600;
  color: var(--ui-text-primary);
}

.latest-ver {
  color: var(--ui-accent) !important;
}

.release-notes {
  font-size: 13px;
  color: var(--ui-text-secondary);
  max-height: 150px;
  overflow-y: auto;
}

.release-notes h4 {
  font-size: 14px;
  margin-bottom: 4px;
  color: var(--ui-text-primary);
}

.dialog-footer {
  display: flex;
  gap: var(--ui-space-sm);
  justify-content: flex-end;
}

.btn {
  padding: 8px 20px;
  border: none;
  border-radius: var(--ui-radius-md);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--ui-transition-fast);
  display: inline-flex;
  align-items: center;
  gap: var(--ui-space-xs);
}

.btn-primary {
  background-color: var(--ui-accent);
  color: var(--ui-text-on-accent);
}

.btn-primary:hover:not(:disabled) {
  background-color: var(--ui-accent-hover);
}

.btn-secondary {
  background-color: var(--ui-bg-secondary);
  color: var(--ui-text-primary);
  border: 1px solid var(--ui-border);
}

.btn-secondary:hover:not(:disabled) {
  background-color: var(--ui-bg-primary);
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@keyframes dialog-fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

@keyframes dialog-fade-out {
  from {
    opacity: 1;
  }
  to {
    opacity: 0;
  }
}
</style>
