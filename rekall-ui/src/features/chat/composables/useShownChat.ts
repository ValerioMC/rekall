import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useAttentionTitle } from '@/features/chat/composables/useAttentionTitle'
import { useChatEvents } from '@/features/chat/composables/useChatEvents'
import { chatStatus } from '@/features/chat/lib/chat-status'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useConsoleStore } from '@/stores/console.store'
import type { Terminal } from '@/features/claude/model/terminal'

/** The chat on screen, what it is doing and the steps it can name. It holds that chat's event stream. */
export function useShownChat() {
  const chat = useChatStore()
  const terminals = useTerminalStore()
  const { selectedTask, selectedTaskSteps } = storeToRefs(useConsoleStore())
  const { activeChatId, activeTimeline } = storeToRefs(chat)
  const { connected, reconnecting } = useChatEvents(activeChatId)

  const session = computed<Terminal | null>(
    () => terminals.terminals.find((terminal) => terminal.id === activeChatId.value) ?? null
  )
  const others = computed(() => chat.liveChats.filter((terminal) => terminal.id !== activeChatId.value))
  const status = computed(() =>
    chatStatus({
      timeline: activeTimeline.value,
      session: session.value,
      connected: connected.value,
      reconnecting: reconnecting.value
    })
  )
  const sessionSteps = computed(() =>
    session.value?.taskId && session.value.taskId === selectedTask.value?.id
      ? selectedTaskSteps.value.filter((step) => step.state !== 'DRAFT')
      : []
  )

  useAttentionTitle(() => status.value === 'waiting')

  return { session, others, status, sessionSteps }
}
