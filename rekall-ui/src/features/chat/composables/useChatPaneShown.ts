import { computed, type ComputedRef } from 'vue'
import { useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'

/** Whether a full-size chat is what is on screen: a task's, or one on a project alone. */
export function useChatPaneShown(): ComputedRef<boolean> {
  const router = useRouter()
  const store = useConsoleStore()
  return computed(() => router.currentRoute.value.name === 'console' && store.chatInPane)
}
