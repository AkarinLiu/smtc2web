import { defineStore } from 'pinia'
import { ref } from 'vue'

export type ToastType = 'success' | 'error' | 'warning' | 'info' | 'confirm'
export type ToastPosition = 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left' | 'top-center' | 'bottom-center'

export interface ToastActions {
  confirmText: string
  cancelText: string
  onConfirm: () => void
  onCancel?: () => void
}

export interface Toast {
  id: string
  type: ToastType
  message: string
  title?: string
  duration: number
  actions?: ToastActions
}

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<Toast[]>([])
  const position = ref<ToastPosition>('top-right')
  const defaultDuration = ref(5000)

  let toastIdCounter = 0


  function generateId(): string {
    return `toast-${Date.now()}-${++toastIdCounter}`
  }

  function addToast(
    message: string,
    type: ToastType = 'info',
    options: {
      title?: string
      duration?: number
      actions?: ToastActions
    } = {}
  ): string {
    const id = generateId()
    // confirm 类型的 toast 不会自动消失
    const duration = type === 'confirm' ? 0 : (options.duration ?? defaultDuration.value)

    const toast: Toast = {
      id,
      type,
      message,
      title: options.title,
      duration,
      actions: options.actions
    }

    toasts.value.push(toast)

    return id
  }

  function removeToast(id: string) {
    const index = toasts.value.findIndex(t => t.id === id)
    if (index > -1) {
      toasts.value.splice(index, 1)
    }
  }

  function clearAll() {
    // 取消所有确认类型的 toast
    toasts.value.forEach(toast => {
      if (toast.type === 'confirm' && toast.actions?.onCancel) {
        toast.actions.onCancel()
      }
    })
    toasts.value = []
  }

  function confirmToast(id: string, confirmed: boolean) {
    const index = toasts.value.findIndex(t => t.id === id)
    if (index > -1) {
      const toast = toasts.value[index]
      if (toast.type === 'confirm' && toast.actions) {
        if (confirmed) {
          toast.actions.onConfirm()
        } else if (toast.actions.onCancel) {
          toast.actions.onCancel()
        }
      }
      toasts.value.splice(index, 1)
    }
  }

  // 便捷方法
  function success(message: string, title?: string, duration?: number) {
    return addToast(message, 'success', { title, duration })
  }

  function error(message: string, title?: string, duration?: number) {
    return addToast(message, 'error', { title, duration })
  }

  function warning(message: string, title?: string, duration?: number) {
    return addToast(message, 'warning', { title, duration })
  }

  function info(message: string, title?: string, duration?: number) {
    return addToast(message, 'info', { title, duration })
  }

  function confirm(
    message: string,
    actions: {
      confirmText: string
      cancelText: string
      onConfirm: () => void
      onCancel?: () => void
    },
    title?: string
  ): string {
    // 检查是否已存在 confirm 类型的 toast，防止重复弹窗
    const existingConfirm = toasts.value.find(t => t.type === 'confirm')
    if (existingConfirm) {
      return existingConfirm.id
    }

    return addToast(message, 'confirm', {
      title,
      actions: {
        confirmText: actions.confirmText,
        cancelText: actions.cancelText,
        onConfirm: actions.onConfirm,
        onCancel: actions.onCancel
      }
    })
  }

  return {
    toasts,
    position,
    defaultDuration,
    addToast,
    removeToast,
    clearAll,
    confirmToast,
    success,
    error,
    warning,
    info,
    confirm,
  }
})
