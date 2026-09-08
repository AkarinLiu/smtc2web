<template>
  <ToastProvider
    :swipe-direction="'right'"
    :duration="defaultDuration"
    :label="'Notifications'"
  >
    <ToastViewport class="toast-container" :class="position">
      <ToastRoot
        v-for="toast in toasts"
        :key="toast.id"
        :duration="toast.duration"
        @update:open="(v) => { if (!v) removeToast(toast.id) }"
      >
        <div
          class="toast"
          :class="[toast.type, { 'is-confirm': toast.type === 'confirm' }]"
        >
          <div class="toast-content">
            <div class="toast-icon">
              <font-awesome-icon :icon="getIcon(toast.type)" />
            </div>
            <div class="toast-message">
              <ToastTitle v-if="toast.title" class="toast-title">{{
                toast.title
              }}</ToastTitle>
              <ToastDescription class="toast-text">{{
                toast.message
              }}</ToastDescription>

              <!-- 确认按钮组 -->
              <div
                v-if="toast.type === 'confirm' && toast.actions"
                class="toast-actions"
              >
                <button
                  class="toast-btn toast-btn-confirm"
                  @click="confirmToast(toast.id, true)"
                >
                  {{ toast.actions.confirmText }}
                </button>
                <button
                  class="toast-btn toast-btn-cancel"
                  @click="confirmToast(toast.id, false)"
                >
                  {{ toast.actions.cancelText }}
                </button>
              </div>
            </div>
          </div>

          <ToastClose
            v-if="toast.type !== 'confirm'"
            class="toast-close"
            :aria-label="'Close'"
          >
            <font-awesome-icon icon="times" />
          </ToastClose>

          <div
            v-if="toast.duration > 0 && toast.type !== 'confirm'"
            class="toast-progress"
            :style="{ animationDuration: `${toast.duration}ms` }"
          />
        </div>
      </ToastRoot>
    </ToastViewport>
  </ToastProvider>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia'
import {
  ToastClose,
  ToastDescription,
  ToastProvider,
  ToastRoot,
  ToastTitle,
  ToastViewport,
} from 'reka-ui'
import { useToastStore } from '@/stores/toast'

const toastStore = useToastStore()
const { toasts, position, defaultDuration } = storeToRefs(toastStore)
const { removeToast, confirmToast } = toastStore

function getIcon(type: string): string {
  switch (type) {
    case 'success':
      return 'check-circle'
    case 'error':
      return 'exclamation-circle'
    case 'warning':
    case 'confirm':
      return 'exclamation-triangle'
    case 'info':
    default:
      return 'info-circle'
  }
}
</script>

<style scoped>
.toast-container {
  position: fixed;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 20px;
  pointer-events: none;
  list-style: none;
}

.toast-container.top-right {
  top: 20px;
  right: 20px;
}

.toast-container.top-left {
  top: 20px;
  left: 20px;
}

.toast-container.bottom-right {
  bottom: 20px;
  right: 20px;
  flex-direction: column-reverse;
}

.toast-container.bottom-left {
  bottom: 20px;
  left: 20px;
  flex-direction: column-reverse;
}

.toast-container.top-center {
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
}

.toast-container.bottom-center {
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  flex-direction: column-reverse;
}

.toast {
  position: relative;
  display: flex;
  align-items: flex-start;
  gap: 12px;
  min-width: 300px;
  max-width: 400px;
  padding: 16px;
  background: var(--ui-bg-card);
  border-radius: var(--ui-radius-lg);
  box-shadow: var(--ui-shadow-lg);
  pointer-events: auto;
  overflow: hidden;
  border-left: 4px solid transparent;
}

.toast.success {
  border-left-color: var(--ui-success);
}

.toast.error {
  border-left-color: var(--ui-error);
}

.toast.warning,
.toast.confirm {
  border-left-color: var(--ui-warning);
}

.toast.info {
  border-left-color: var(--ui-accent);
}

.toast-content {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  flex: 1;
}

.toast-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  flex-shrink: 0;
  font-size: 20px;
}

.toast.success .toast-icon {
  color: var(--ui-success);
}

.toast.error .toast-icon {
  color: var(--ui-error);
}

.toast.warning .toast-icon,
.toast.confirm .toast-icon {
  color: var(--ui-warning);
}

.toast.info .toast-icon {
  color: var(--ui-accent);
}

.toast-message {
  flex: 1;
  min-width: 0;
}

.toast-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--ui-text-primary);
  margin-bottom: 4px;
}

.toast-text {
  font-size: 13px;
  color: var(--ui-text-secondary);
  line-height: 1.5;
  word-wrap: break-word;
}

.toast-actions {
  display: flex;
  gap: 8px;
  margin-top: 12px;
}

.toast-btn {
  padding: 6px 12px;
  border: none;
  border-radius: var(--ui-radius-md);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.toast-btn-confirm {
  background-color: var(--ui-error);
  color: white;
}

.toast-btn-confirm:hover {
  background-color: var(--ui-error-hover);
}

.toast-btn-cancel {
  background-color: var(--ui-bg-secondary);
  color: var(--ui-text-primary);
}

.toast-btn-cancel:hover {
  background-color: var(--ui-bg-tertiary);
}

.toast-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  padding: 0;
  background: transparent;
  border: none;
  color: var(--ui-text-secondary);
  cursor: pointer;
  border-radius: var(--ui-radius-sm);
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.toast-close:hover {
  background: var(--ui-bg-secondary);
  color: var(--ui-text-primary);
}

.toast-progress {
  position: absolute;
  bottom: 0;
  left: 0;
  height: 3px;
  background: currentColor;
  opacity: 0.3;
  animation: progress linear forwards;
}

.toast.success .toast-progress {
  background: var(--ui-success);
}

.toast.error .toast-progress {
  background: var(--ui-error);
}

.toast.warning .toast-progress {
  background: var(--ui-warning);
}

.toast.info .toast-progress {
  background: var(--ui-accent);
}

@keyframes progress {
  from {
    width: 100%;
  }
  to {
    width: 0%;
  }
}

/* 响应式 */
@media (max-width: 480px) {
  .toast-container {
    left: 10px !important;
    right: 10px !important;
    top: 10px !important;
    bottom: auto !important;
    transform: none !important;
    padding: 10px;
  }

  .toast {
    min-width: auto;
    max-width: none;
    width: 100%;
  }
}
</style>
