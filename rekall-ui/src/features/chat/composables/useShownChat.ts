import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { chatStatus } from '@/features/chat/lib/chat-status'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useConsoleStore } from '@/stores/console.store'

/** The chat on screen, what it is doing and the steps it can name. */
export function useShownChat() {
  const chat = useChatStore()
  const { selectedTask, selectedTaskSteps } = storeToRefs(useConsoleStore())
  const { activeChatId, activeSession: session, activeTimeline, connection } = storeToRefs(chat)

  const others = computed(() => chat.liveChats.filter((terminal) => terminal.id !== activeChatId.value))
  const status = computed(() =>
    chatStatus({ timeline: activeTimeline.value, session: session.value, ...connection.value })
  )
  const sessionSteps = computed(() =>
    session.value?.taskId && session.value.taskId === selectedTask.value?.id
      ? selectedTaskSteps.value.filter((step) => step.state !== 'DRAFT')
      : []
  )

  return { session, others, status, sessionSteps }
}
