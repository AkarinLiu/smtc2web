<template>
  <Teleport to="body">
    <Transition name="dialog-fade">
      <div v-if="updateStore.showDialog" class="dialog-overlay" @click.self="updateStore.closeDialog">
        <div class="dialog-content">
          <div class="dialog-header">
            <font-awesome-icon icon="circle-up" class="dialog-icon" />
            <h3>{{ t('update.title') }}</h3>
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

            <button
              class="btn btn-secondary"
              @click="updateStore.closeDialog"
            >
              {{ lastResult?.has_update ? t('update.later') : t('common.confirm') }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useUpdateStore } from '@/stores/update'

const { t } = useI18n()
const updateStore = useUpdateStore()

const lastResult = computed(() => updateStore.lastResult)

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
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.dialog-content {
  background: var(--fluent-bg-card);
  border-radius: var(--fluent-radius-lg);
  box-shadow: var(--fluent-shadow-lg);
  max-width: 440px;
  width: 90%;
  padding: var(--fluent-space-lg);
}

.dialog-header {
  display: flex;
  align-items: center;
  gap: var(--fluent-space-sm);
  margin-bottom: var(--fluent-space-lg);
}

.dialog-icon {
  font-size: 24px;
  color: var(--fluent-accent);
}

.dialog-header h3 {
  margin: 0;
  font-size: 18px;
  color: var(--fluent-text-primary);
}

.dialog-body {
  margin-bottom: var(--fluent-space-lg);
}

.dialog-error {
  color: #dc2626;
  font-size: 14px;
}

.dialog-no-update {
  color: #16a34a;
  font-size: 14px;
}

.version-info {
  background: var(--fluent-bg-secondary);
  border-radius: var(--fluent-radius-md);
  padding: var(--fluent-space-md);
  margin-bottom: var(--fluent-space-md);
}

.version-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 0;
}

.version-row .label {
  font-size: 13px;
  color: var(--fluent-text-secondary);
}

.version-row code {
  font-size: 14px;
  font-weight: 600;
  color: var(--fluent-text-primary);
}

.latest-ver {
  color: var(--fluent-accent) !important;
}

.release-notes {
  font-size: 13px;
  color: var(--fluent-text-secondary);
  max-height: 150px;
  overflow-y: auto;
}

.release-notes h4 {
  font-size: 14px;
  margin-bottom: 4px;
  color: var(--fluent-text-primary);
}

.dialog-footer {
  display: flex;
  gap: var(--fluent-space-sm);
  justify-content: flex-end;
}

.btn {
  padding: 8px 20px;
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

.btn-secondary {
  background-color: var(--fluent-bg-secondary);
  color: var(--fluent-text-primary);
  border: 1px solid var(--fluent-border);
}

.btn-secondary:hover:not(:disabled) {
  background-color: var(--fluent-bg-primary);
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.dialog-fade-enter-active,
.dialog-fade-leave-active {
  transition: opacity 0.2s ease;
}

.dialog-fade-enter-from,
.dialog-fade-leave-to {
  opacity: 0;
}
</style>
