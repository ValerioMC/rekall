import { defineStore } from 'pinia'
import { ref } from 'vue'
import { ApiError } from '@/api/client'

export type ToastKind = 'success' | 'error'

export type Toast = Readonly<{ id: number; message: string; kind: ToastKind }>

const DURATION_MS: Readonly<Record<ToastKind, number>> = { success: 3200, error: 7000 }

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<readonly Toast[]>([])
  let nextId = 0

  function push(message: string, kind: ToastKind): void {
    const toast: Toast = { id: nextId++, message, kind }
    toasts.value = [...toasts.value, toast]
    window.setTimeout(() => dismiss(toast.id), DURATION_MS[kind])
  }

  function dismiss(id: number): void {
    toasts.value = toasts.value.filter((toast) => toast.id !== id)
  }

  function notify(message: string): void {
    push(message, 'success')
  }

  function notifyError(error: unknown): void {
    if (error instanceof ApiError) {
      push(error.message, 'error')
      return
    }
    push(error instanceof Error ? error.message : String(error), 'error')
  }

  return { toasts, notify, notifyError, dismiss }
})
