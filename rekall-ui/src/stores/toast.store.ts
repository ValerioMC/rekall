import { defineStore } from 'pinia'
import { ref } from 'vue'
import { ApiError } from '@/api/client'

export type ToastKind = 'success' | 'error'

export type Toast = Readonly<{ id: number; message: string; kind: ToastKind }>

/** Errors stay up longer: the server's messages explain what to do instead of what failed. */
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

  /** Surfaces the server's own wording, which is written to be read by the person who hit it. */
  function notifyError(error: unknown): void {
    if (error instanceof ApiError) {
      push(error.message, 'error')
      return
    }
    push(error instanceof Error ? error.message : String(error), 'error')
  }

  return { toasts, notify, notifyError, dismiss }
})
