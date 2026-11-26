import { computed, type ComputedRef } from 'vue'
import { useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'

/** Whether the full-size terminal of a task is what is on screen. */
export function useTerminalPaneShown(): ComputedRef<boolean> {
  const router = useRouter()
  const store = useConsoleStore()
  return computed(
    () => router.currentRoute.value.name === 'console' && store.paneFocus === 'terminal' && store.selectedTask !== null
  )
}
