<template>
  <Teleport to="body">
    <TransitionGroup
      name="toast"
      tag="div"
      class="toast-container"
      :class="position"
    >
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast"
        :class="[toast.type, { 
          'has-progress': toast.duration > 0 && toast.type !== 'confirm',
          'is-confirm': toast.type === 'confirm'
        }]"
        @mouseenter="pauseToast(toast.id)"
        @mouseleave="resumeToast(toast.id)"
      >
        <div class="toast-content">
          <div class="toast-icon">
            <font-awesome-icon :icon="getIcon(toast.type)" />
          </div>
          <div class="toast-message">
            <div v-if="toast.title" class="toast-title">{{ toast.title }}</div>
            <div class="toast-text">{{ toast.message }}</div>
            
            <!-- 确认按钮组 -->
            <div v-if="toast.type === 'confirm' && toast.actions" class="toast-actions">
              <button 
                class="toast-btn toast-btn-confirm" 
                @click="handleConfirm(toast.id, true)"
              >
                {{ toast.actions.confirmText }}
              </button>
              <button 
                class="toast-btn toast-btn-cancel" 
                @click="handleConfirm(toast.id, false)"
              >
                {{ toast.actions.cancelText }}
              </button>
            </div>
          </div>
        </div>
        
        <button 
          v-if="toast.type !== 'confirm'" 
          class="toast-close" 
          @click="removeToast(toast.id)"
        >
          <font-awesome-icon icon="times" />
        </button>
        
        <div
          v-if="toast.duration > 0 && toast.type !== 'confirm'"
          class="toast-progress"
          :style="{ animationDuration: `${toast.duration}ms` }"
        />
      </div>
    </TransitionGroup>
  </Teleport>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useToastStore } from '@/stores/toast'

const toastStore = useToastStore()
const { toasts, position } = storeToRefs(toastStore)
const { removeToast, pauseToast, resumeToast, confirmToast } = toastStore

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

function handleConfirm(id: string, confirmed: boolean) {
  confirmToast(id, confirmed)
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
  background: var(--fluent-bg-card);
  border-radius: var(--fluent-radius-lg);
  box-shadow: var(--fluent-shadow-lg);
  pointer-events: auto;
  overflow: hidden;
  border-left: 4px solid transparent;
}

.toast.success {
  border-left-color: var(--fluent-success);
}

.toast.error {
  border-left-color: var(--fluent-error);
}

.toast.warning,
.toast.confirm {
  border-left-color: var(--fluent-warning);
}

.toast.info {
  border-left-color: var(--fluent-accent);
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
  color: var(--fluent-success);
}

.toast.error .toast-icon {
  color: var(--fluent-error);
}

.toast.warning .toast-icon,
.toast.confirm .toast-icon {
  color: var(--fluent-warning);
}

.toast.info .toast-icon {
  color: var(--fluent-accent);
}

.toast-message {
  flex: 1;
  min-width: 0;
}

.toast-title {
  font-weight: 600;
  font-size: 14px;
  color: var(--fluent-text-primary);
  margin-bottom: 4px;
}

.toast-text {
  font-size: 13px;
  color: var(--fluent-text-secondary);
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
  border-radius: var(--fluent-radius-md);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.toast-btn-confirm {
  background-color: var(--fluent-error);
  color: white;
}

.toast-btn-confirm:hover {
  background-color: var(--fluent-error-hover, #d32f2f);
}

.toast-btn-cancel {
  background-color: var(--fluent-bg-secondary);
  color: var(--fluent-text-primary);
}

.toast-btn-cancel:hover {
  background-color: var(--fluent-bg-tertiary);
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
  color: var(--fluent-text-secondary);
  cursor: pointer;
  border-radius: var(--fluent-radius-sm);
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.toast-close:hover {
  background: var(--fluent-bg-secondary);
  color: var(--fluent-text-primary);
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
  background: var(--fluent-success);
}

.toast.error .toast-progress {
  background: var(--fluent-error);
}

.toast.warning .toast-progress {
  background: var(--fluent-warning);
}

.toast.info .toast-progress {
  background: var(--fluent-accent);
}

@keyframes progress {
  from {
    width: 100%;
  }
  to {
    width: 0%;
  }
}

/* 过渡动画 */
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(100%);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(100%);
}

.toast-container.top-left .toast-enter-from,
.toast-container.bottom-left .toast-enter-from {
  transform: translateX(-100%);
}

.toast-container.top-left .toast-leave-to,
.toast-container.bottom-left .toast-leave-to {
  transform: translateX(-100%);
}

.toast-container.top-center .toast-enter-from,
.toast-container.bottom-center .toast-enter-from {
  transform: translateY(-20px) scale(0.9);
}

.toast-container.top-center .toast-leave-to,
.toast-container.bottom-center .toast-leave-to {
  transform: translateY(-20px) scale(0.9);
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
