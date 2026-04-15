import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

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
  paused: boolean
  remainingTime: number
  actions?: ToastActions
}

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<Toast[]>([])
  const position = ref<ToastPosition>('top-right')
  const defaultDuration = ref(5000)

  let toastIdCounter = 0
  const timers = new Map<string, number>()

  const toastCount = computed(() => toasts.value.length)

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
      paused: false,
      remainingTime: duration,
      actions: options.actions
    }

    toasts.value.push(toast)

    if (duration > 0) {
      startTimer(id, duration)
    }

    return id
  }

  function removeToast(id: string) {
    const index = toasts.value.findIndex(t => t.id === id)
    if (index > -1) {
      const toast = toasts.value[index]
      // 如果是确认类型的 toast，取消时执行 onCancel 回调
      if (toast.type === 'confirm' && toast.actions?.onCancel) {
        toast.actions.onCancel()
      }
      toasts.value.splice(index, 1)
      clearTimer(id)
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
    timers.forEach((_, id) => clearTimer(id))
    timers.clear()
  }

  function startTimer(id: string, duration: number) {
    clearTimer(id)
    const timerId = window.setTimeout(() => {
      removeToast(id)
    }, duration)
    timers.set(id, timerId)
  }

  function clearTimer(id: string) {
    const timerId = timers.get(id)
    if (timerId) {
      window.clearTimeout(timerId)
      timers.delete(id)
    }
  }

  function pauseToast(id: string) {
    const toast = toasts.value.find(t => t.id === id)
    if (toast && !toast.paused && toast.duration > 0) {
      toast.paused = true
      clearTimer(id)
    }
  }

  function resumeToast(id: string) {
    const toast = toasts.value.find(t => t.id === id)
    if (toast && toast.paused && toast.duration > 0) {
      toast.paused = false
      startTimer(id, toast.duration)
    }
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
      clearTimer(id)
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

  function setPosition(newPosition: ToastPosition) {
    position.value = newPosition
  }

  return {
    toasts,
    position,
    toastCount,
    addToast,
    removeToast,
    clearAll,
    pauseToast,
    resumeToast,
    confirmToast,
    success,
    error,
    warning,
    info,
    confirm,
    setPosition
  }
})
