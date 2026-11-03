import { computed, type ComputedRef } from 'vue'
import { storeToRefs } from 'pinia'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'
import { useConsoleStore } from '@/stores/console.store'

/** Whether the chat window is on screen: not hidden, and the selected task has a session to show or asked for the window. */
export function useChatWindowShown(): ComputedRef<boolean> {
  const chat = useChatStore()
  const dock = useChatDockStore()
  const { selectedTaskId } = storeToRefs(useConsoleStore())
  return computed(() => {
    if (dock.hidden) return false
    const taskId = selectedTaskId.value
    return chat.sessionFor(taskId) !== null || (taskId !== null && dock.requestedFor === taskId)
  })
}
