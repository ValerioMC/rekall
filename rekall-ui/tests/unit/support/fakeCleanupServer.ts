import { vi } from 'vitest'
import type { PictureCleanupStatus, PictureCleanupStreamEvent } from '@/features/picture/model/picture-cleanup'

type Listener = (event: unknown) => void

class FakeEventSource {
  static open: FakeEventSource | null = null
  readonly readyState = 1
  private readonly listeners = new Map<string, Listener[]>()

  constructor() {
    FakeEventSource.open = this
  }

  addEventListener(type: string, listener: Listener): void {
    this.listeners.set(type, [...(this.listeners.get(type) ?? []), listener])
  }

  emit(type: string, event: unknown): void {
    for (const listener of this.listeners.get(type) ?? []) listener(event)
  }

  close(): void {}
}

/**
 * Stands in for the cleanup routes and their frames: `status` is what the next read answers,
 * `afterRequest` what a write turns it into, and `requests` lists what was asked.
 */
export function fakeCleanupServer(initial: PictureCleanupStatus = { state: 'IDLE' }) {
  const server = {
    status: initial,
    requests: [] as string[],
    unreadable: false,
    afterRequest: {} as Record<string, PictureCleanupStatus>,
    emit(frame: PictureCleanupStreamEvent): void {
      FakeEventSource.open?.emit('picture-cleanup', { data: JSON.stringify(frame) })
    }
  }
  vi.stubGlobal('EventSource', FakeEventSource)
  vi.stubGlobal('fetch', async (url: string, init?: RequestInit) => {
    const request = `${init?.method ?? 'GET'} ${url}`
    server.requests.push(request)
    if (request.startsWith('GET')) {
      if (server.unreadable) return new Response(JSON.stringify({ detail: 'Not now.' }), { status: 404, headers: { 'content-type': 'application/json' } })
      return new Response(JSON.stringify(server.status), { headers: { 'content-type': 'application/json' } })
    }
    server.status = server.afterRequest[request] ?? server.status
    return new Response(null, { status: request.startsWith('DELETE') ? 204 : 202 })
  })
  return server
}
