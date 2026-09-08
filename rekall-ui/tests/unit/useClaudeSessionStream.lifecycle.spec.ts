import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { effectScope, nextTick, ref } from 'vue'
import { useClaudeSessionStream } from '@/composables/useClaudeSessionStream'
import type { ClaudeSessionId } from '@/model/branded'

/**
 * The session pane follows one hosted session at a time. When the pane switches sessions the
 * previous `EventSource` must be closed before the next opens, and the last one closed with the
 * scope. A fan of connections left open is the leak this guards, the same discipline
 * {@link useStepStream.lifecycle.spec.ts} keeps on the console feed.
 */

class FakeEventSource {
  static instances: FakeEventSource[] = []
  static readonly OPEN = 1
  static readonly CONNECTING = 0

  readyState = FakeEventSource.CONNECTING
  closed = false
  private readonly listeners = new Map<string, ((event: unknown) => void)[]>()

  constructor(readonly url: string) {
    FakeEventSource.instances.push(this)
  }

  addEventListener(type: string, listener: (event: unknown) => void): void {
    const bucket = this.listeners.get(type) ?? []
    bucket.push(listener)
    this.listeners.set(type, bucket)
  }

  emit(type: string, event: unknown): void {
    if (this.closed) return
    for (const listener of this.listeners.get(type) ?? []) listener(event)
  }

  close(): void {
    this.closed = true
  }
}

const handlers = () => ({ onMessage: vi.fn(), onStatus: vi.fn(), onEnded: vi.fn() })

describe('useClaudeSessionStream lifecycle', () => {
  beforeEach(() => {
    FakeEventSource.instances = []
    vi.stubGlobal('EventSource', FakeEventSource)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('opens no connection while the session id is null', () => {
    const scope = effectScope()
    scope.run(() => useClaudeSessionStream(ref(null), handlers()))

    expect(FakeEventSource.instances).toHaveLength(0)
    scope.stop()
  })

  it('closes the previous connection when the session id changes', async () => {
    const id = ref<ClaudeSessionId | null>('11111111-1111-1111-1111-111111111111' as ClaudeSessionId)
    const scope = effectScope()
    scope.run(() => useClaudeSessionStream(id, handlers()))

    expect(FakeEventSource.instances).toHaveLength(1)
    expect(FakeEventSource.instances[0]!.url).toContain(
      '/api/claude/sessions/11111111-1111-1111-1111-111111111111/stream'
    )

    id.value = '22222222-2222-2222-2222-222222222222' as ClaudeSessionId
    await nextTick()

    expect(FakeEventSource.instances).toHaveLength(2)
    expect(FakeEventSource.instances[0]!.closed).toBe(true)
    expect(FakeEventSource.instances[1]!.closed).toBe(false)

    scope.stop()
    expect(FakeEventSource.instances[1]!.closed).toBe(true)
  })

  it('routes a message frame to onMessage and stops after the scope is disposed', () => {
    const h = handlers()
    const id = ref<ClaudeSessionId | null>('33333333-3333-3333-3333-333333333333' as ClaudeSessionId)
    const scope = effectScope()
    scope.run(() => useClaudeSessionStream(id, h))
    const source = FakeEventSource.instances[0]!

    source.emit('message', {
      data: JSON.stringify({
        id: '44444444-4444-4444-4444-444444444444',
        sessionId: '33333333-3333-3333-3333-333333333333',
        seq: 0,
        role: 'ASSISTANT',
        content: 'hi',
        toolName: null,
        meta: null,
        createdAt: '2026-09-08T10:00:00Z'
      })
    })
    expect(h.onMessage).toHaveBeenCalledTimes(1)

    scope.stop()
    source.emit('message', { data: '{}' })
    expect(h.onMessage).toHaveBeenCalledTimes(1)
  })

  it('drops a frame that does not parse', () => {
    const h = handlers()
    const scope = effectScope()
    scope.run(() =>
      useClaudeSessionStream(
        ref('55555555-5555-5555-5555-555555555555' as ClaudeSessionId),
        h
      )
    )

    expect(() =>
      FakeEventSource.instances[0]!.emit('status', { data: '{ not json' })
    ).not.toThrow()
    expect(h.onStatus).not.toHaveBeenCalled()
    scope.stop()
  })
})
