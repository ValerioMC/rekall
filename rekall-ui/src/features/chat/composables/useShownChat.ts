import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { chatStatus } from '@/features/chat/lib/chat-status'
import { useChatStore } from '@/features/chat/store/chat.store'

/** The chat on screen, what it is doing and the other live chats. */
export function useShownChat() {
  const chat = useChatStore()
  const { activeChatId, activeSession: session, activeTimeline, connection } = storeToRefs(chat)

  const others = computed(() => chat.liveChats.filter((terminal) => terminal.id !== activeChatId.value))
  const status = computed(() =>
    chatStatus({ timeline: activeTimeline.value, session: session.value, ...connection.value })
  )

  return { session, others, status }
}
