import { onScopeDispose } from 'vue'
import { env } from '@/common/config/env'
import { TokenUsageStreamEventSchema } from '@/features/tokens/api/schemas/token-usage.schema'
import type { TokenUsageStreamEvent } from '@/features/tokens/model/token-usage'

/** The `token-usage` frames of the console's event stream: a task's tallies after a transcript read changed them. */
export function useTokenUsageStream(onChange: (event: TokenUsageStreamEvent) => void): void {
  if (typeof EventSource === 'undefined') return
  const source = new EventSource(`${env.VITE_API_BASE_URL}/api/steps/stream`)
  source.addEventListener('token-usage', (event) => {
    let payload: unknown
    try {
      payload = JSON.parse((event as MessageEvent<string>).data)
    } catch {
      return
    }
    const parsed = TokenUsageStreamEventSchema.safeParse(payload)
    if (parsed.success) onChange(parsed.data)
  })
  onScopeDispose(() => source.close())
}
