import { computed, type ComputedRef } from 'vue'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useChatDockStore } from '@/features/chat/store/chatDock.store'

/** Whether the chat window is on screen: not hidden, and there is a session to show or someone asked for it. */
export function useChatWindowShown(): ComputedRef<boolean> {
  const chat = useChatStore()
  const dock = useChatDockStore()
  return computed(
    () => !dock.hidden && (chat.activeChatId !== null || chat.liveChats.length > 0 || dock.requested)
  )
}
