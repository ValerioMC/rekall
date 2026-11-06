import { onScopeDispose } from 'vue'
import { subscribeToFrame } from '@/common/stream/consoleEventStream'
import { TokenUsageStreamEventSchema } from '@/features/tokens/api/schemas/token-usage.schema'
import type { TokenUsageStreamEvent } from '@/features/tokens/model/token-usage'

/** The `token-usage` frames of the console's event stream: a task's tallies after a transcript read changed them. */
export function useTokenUsageStream(onChange: (event: TokenUsageStreamEvent) => void): void {
  const unsubscribe = subscribeToFrame('token-usage', (event) => {
    let payload: unknown
    try {
      payload = JSON.parse((event as MessageEvent<string>).data)
    } catch {
      return
    }
    const parsed = TokenUsageStreamEventSchema.safeParse(payload)
    if (parsed.success) onChange(parsed.data)
  })
  onScopeDispose(unsubscribe)
}
