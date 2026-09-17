import { defineStore } from 'pinia'
import { ref } from 'vue'
import { ApiError } from '@/api/client'

export type ToastKind = 'success' | 'error'

/** One thing the reader can do from the toast, such as undo what it announces. */
export type ToastAction = Readonly<{ label: string; run: () => void }>

export type Toast = Readonly<{ id: number; message: string; kind: ToastKind; action: ToastAction | null }>

const DURATION_MS: Readonly<Record<ToastKind, number>> = { success: 3200, error: 7000 }

/** A toast that offers something to do stays long enough to reach for it. */
const ACTION_DURATION_MS = 6000

export const useToastStore = defineStore('toast', () => {
  const toasts = ref<readonly Toast[]>([])
  let nextId = 0

  function push(message: string, kind: ToastKind, action: ToastAction | null = null): void {
    const toast: Toast = { id: nextId++, message, kind, action }
    toasts.value = [...toasts.value, toast]
    window.setTimeout(() => dismiss(toast.id), action ? ACTION_DURATION_MS : DURATION_MS[kind])
  }

  function dismiss(id: number): void {
    toasts.value = toasts.value.filter((toast) => toast.id !== id)
  }

  function notify(message: string, action?: ToastAction): void {
    push(message, 'success', action ?? null)
  }

  /** Runs the toast's action once and takes the toast down with it. */
  function act(id: number): void {
    const toast = toasts.value.find((candidate) => candidate.id === id)
    if (!toast?.action) return
    dismiss(id)
    toast.action.run()
  }

  function notifyError(error: unknown): void {
    if (error instanceof ApiError) {
      push(error.message, 'error')
      return
    }
    push(error instanceof Error ? error.message : String(error), 'error')
  }

  return { toasts, notify, notifyError, dismiss, act }
})
