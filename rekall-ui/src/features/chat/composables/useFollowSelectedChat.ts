import { computed, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useAttentionTitle } from '@/features/chat/composables/useAttentionTitle'
import { useChatEvents } from '@/features/chat/composables/useChatEvents'
import { chatStatus } from '@/features/chat/lib/chat-status'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useConsoleStore } from '@/stores/console.store'

/**
 * Follows the selected task's chat, on screen or not, so an approval is heard anywhere. It holds
 * the only chat event stream: mount it once.
 */
export function useFollowSelectedChat(): void {
  const chat = useChatStore()
  const { selectedTaskId } = storeToRefs(useConsoleStore())
  const { activeChatId, activeSession, activeTimeline } = storeToRefs(chat)
  const { connected, reconnecting } = useChatEvents(activeChatId)
  const status = computed(() =>
    chatStatus({
      timeline: activeTimeline.value,
      session: activeSession.value,
      connected: connected.value,
      reconnecting: reconnecting.value
    })
  )

  watch(() => chat.sessionFor(selectedTaskId.value)?.id ?? null, chat.select, { immediate: true })
  watch(
    [connected, reconnecting],
    ([isConnected, isReconnecting]) => chat.noteConnection({ connected: isConnected, reconnecting: isReconnecting }),
    { immediate: true }
  )
  useAttentionTitle(() => status.value === 'waiting')
}
