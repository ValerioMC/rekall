import { onScopeDispose } from 'vue'
import { subscribeToFrame } from '@/common/stream/consoleEventStream'
import { ConnectionStreamEventSchema } from '@/features/connection/api/schemas/connection.schema'
import type { ConnectionStreamEvent } from '@/features/connection/model/connection'

/**
 * The `connection` frames of the console's event stream: a change made in another window, or a
 * binding made from a project page, redraws an open Connections screen without a reload.
 */
export function useConnectionStream(onChange: (event: ConnectionStreamEvent) => void): void {
  const unsubscribe = subscribeToFrame('connection', (event) => {
    let payload: unknown
    try {
      payload = JSON.parse((event as MessageEvent<string>).data)
    } catch {
      return
    }
    const parsed = ConnectionStreamEventSchema.safeParse(payload)
    if (parsed.success) onChange(parsed.data)
  })
  onScopeDispose(unsubscribe)
}
