import { onScopeDispose, ref, watch, type Ref } from 'vue'
import { env } from '@/common/config/env'
import { ClaudeMessageSchema, ClaudeSessionSchema } from '@/api/schemas/claude.schema'
import type { ClaudeMessage, ClaudeSession } from '@/model/claude'
import type { ClaudeSessionId } from '@/model/branded'

export interface ClaudeSessionStreamHandlers {
  onMessage: (message: ClaudeMessage) => void
  onStatus: (session: ClaudeSession) => void
  onEnded: (session: ClaudeSession) => void
}

/**
 * Follows exactly one hosted session: the one whose pane is open. When {@link sessionId}
 * changes the previous `EventSource` is closed and a new one opened, and the last one is closed
 * with the calling scope. One connection at a time, never a fan of them left behind: the same
 * discipline `useStepStream` keeps, for the same reason.
 */
export function useClaudeSessionStream(
  sessionId: Ref<ClaudeSessionId | null>,
  handlers: ClaudeSessionStreamHandlers
): { connected: Ref<boolean> } {
  const connected = ref(false)
  let source: EventSource | null = null

  function close(): void {
    source?.close()
    source = null
    connected.value = false
  }

  function open(id: ClaudeSessionId): void {
    close()
    if (typeof EventSource === 'undefined') return
    const next = new EventSource(`${env.VITE_API_BASE_URL}/api/claude/sessions/${id}/stream`)
    source = next

    next.addEventListener('open', () => {
      connected.value = true
    })
    next.addEventListener('error', () => {
      connected.value = next.readyState === EventSource.OPEN
    })
    next.addEventListener('message', (event) => {
      const parsed = safeParse(ClaudeMessageSchema, event)
      if (parsed) handlers.onMessage(parsed)
    })
    next.addEventListener('status', (event) => {
      const parsed = safeParse(ClaudeSessionSchema, event)
      if (parsed) handlers.onStatus(parsed)
    })
    next.addEventListener('ended', (event) => {
      const parsed = safeParse(ClaudeSessionSchema, event)
      if (parsed) handlers.onEnded(parsed)
    })
  }

  watch(
    sessionId,
    (id) => {
      if (id) open(id)
      else close()
    },
    { immediate: true }
  )

  onScopeDispose(close)

  return { connected }
}

function safeParse<T>(
  schema: { safeParse: (value: unknown) => { success: true; data: T } | { success: false } },
  event: Event
): T | null {
  try {
    const result = schema.safeParse(JSON.parse((event as MessageEvent<string>).data))
    return result.success ? result.data : null
  } catch {
    return null
  }
}
