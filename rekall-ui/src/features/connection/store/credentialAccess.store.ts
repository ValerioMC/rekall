import { defineStore } from 'pinia'
import { ref } from 'vue'
import { env } from '@/common/config/env'
import { CredentialAccessEventSchema } from '@/features/connection/api/schemas/connection.schema'
import type { CredentialAccessEvent, CredentialAccessTally } from '@/features/connection/model/connection'
import type { ConnectionId, TerminalId } from '@/model/branded'

/**
 * How often each terminal's helpers asked for each connection since the console opened, read from
 * the `credential-access` frames. One subscription for the whole console, however many chips show it.
 */
export const useCredentialAccessStore = defineStore('credentialAccess', () => {
  const tallies = ref<Readonly<Record<string, CredentialAccessTally>>>({})
  let source: EventSource | null = null

  function key(terminalId: TerminalId, connectionId: ConnectionId): string {
    return `${terminalId}:${connectionId}`
  }

  function tallyOf(terminalId: TerminalId, connectionId: ConnectionId): CredentialAccessTally | null {
    return tallies.value[key(terminalId, connectionId)] ?? null
  }

  function record(event: CredentialAccessEvent): void {
    const id = key(event.terminalId, event.connectionId)
    const before = tallies.value[id] ?? { granted: 0, refused: 0, last: 0 }
    const after = event.granted
      ? { ...before, granted: before.granted + 1, last: before.last + 1 }
      : { ...before, refused: before.refused + 1, last: before.last + 1 }
    tallies.value = { ...tallies.value, [id]: after }
  }

  /** Opens the one subscription, the first time a chip needs it. */
  function listen(): void {
    if (source || typeof EventSource === 'undefined') return
    source = new EventSource(`${env.VITE_API_BASE_URL}/api/steps/stream`)
    source.addEventListener('credential-access', (event) => {
      let payload: unknown
      try {
        payload = JSON.parse((event as MessageEvent<string>).data)
      } catch {
        return
      }
      const parsed = CredentialAccessEventSchema.safeParse(payload)
      if (parsed.success) record(parsed.data)
    })
  }

  return { tallyOf, record, listen }
})
