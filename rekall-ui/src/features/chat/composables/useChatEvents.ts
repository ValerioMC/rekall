import { computed, onScopeDispose, ref, watch, type ComputedRef, type Ref } from 'vue'
import { chatEventsUrl, parseChatEvent } from '@/features/chat/api/chat.api'
import { useChatStore } from '@/features/chat/store/chat.store'
import type { TerminalId } from '@/model/branded'

export interface ChatEventStream {
  connected: Ref<boolean>
  reconnecting: Ref<boolean>
  ended: ComputedRef<boolean>
}

/**
 * The one `EventSource` the chat window holds, on the session it follows. A first connection asks
 * for what came after the last event the store holds; the browser's own reconnects resume with
 * `Last-Event-ID`. Changing session or leaving the scope closes it, and an ended session is not
 * reopened.
 */
export function useChatEvents(sessionId: Ref<TerminalId | null>): ChatEventStream {
  const chat = useChatStore()
  const connected = ref(false)
  const reconnecting = ref(false)
  const ended = computed(() => (sessionId.value ? chat.timelineOf(sessionId.value).ended : false))
  let source: EventSource | null = null

  function close(): void {
    source?.close()
    source = null
    connected.value = false
    reconnecting.value = false
  }

  function open(id: TerminalId): void {
    const next = new EventSource(chatEventsUrl(id, chat.timelineOf(id).lastSeq))
    source = next
    next.onopen = () => {
      connected.value = true
      reconnecting.value = false
    }
    next.onmessage = (message: MessageEvent<string>) => {
      const event = parseChatEvent(message.data)
      if (!event) return
      chat.ingest(id, event)
      if (event.type === 'ended') close()
    }
    next.onerror = () => {
      connected.value = false
      reconnecting.value = next.readyState === EventSource.CONNECTING
    }
  }

  watch(
    sessionId,
    (id) => {
      close()
      if (id && !chat.timelineOf(id).ended) open(id)
    },
    { immediate: true }
  )

  onScopeDispose(close)

  return { connected, reconnecting, ended }
}
