import { onScopeDispose } from 'vue'
import { env } from '@/common/config/env'
import { ConnectionStreamEventSchema } from '@/features/connection/api/schemas/connection.schema'
import type { ConnectionStreamEvent } from '@/features/connection/model/connection'

/**
 * The `connection` frames of the console's event stream: a change made in another window, or a
 * binding made from a project page, redraws an open Connections screen without a reload.
 */
export function useConnectionStream(onChange: (event: ConnectionStreamEvent) => void): void {
  if (typeof EventSource === 'undefined') return
  const source = new EventSource(`${env.VITE_API_BASE_URL}/api/steps/stream`)
  source.addEventListener('connection', (event) => {
    let payload: unknown
    try {
      payload = JSON.parse((event as MessageEvent<string>).data)
    } catch {
      return
    }
    const parsed = ConnectionStreamEventSchema.safeParse(payload)
    if (parsed.success) onChange(parsed.data)
  })
  onScopeDispose(() => source.close())
}
