import { onScopeDispose } from 'vue'
import { subscribeToFrame } from '@/common/stream/consoleEventStream'
import { ContextFillStreamEventSchema } from '@/features/context-fill/api/schemas/context-fill.schema'
import type { ContextFillStreamEvent } from '@/features/context-fill/model/context-fill'

/** The `context-fill` frames of the console's event stream: a task's whole list of fills after a chat turn changed one. */
export function useContextFillStream(onChange: (event: ContextFillStreamEvent) => void): void {
  const unsubscribe = subscribeToFrame('context-fill', (event) => {
    let payload: unknown
    try {
      payload = JSON.parse((event as MessageEvent<string>).data)
    } catch {
      return
    }
    const parsed = ContextFillStreamEventSchema.safeParse(payload)
    if (parsed.success) onChange(parsed.data)
  })
  onScopeDispose(unsubscribe)
}
