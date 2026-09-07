import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { effectScope } from 'vue'
import { useStepStream } from '@/composables/useStepStream'
import type { TaskId } from '@/model/branded'

/**
 * The live feed, exercised against a stand-in EventSource: the console opens one connection,
 * turns a `steps` frame into a call to the store, drops a malformed frame, and closes the
 * connection when its scope is torn down.
 */

interface Listener {
  (event: unknown): void
}

class FakeEventSource {
  static instances: FakeEventSource[] = []
  static readonly OPEN = 1
  static readonly CONNECTING = 0

  readyState = FakeEventSource.CONNECTING
  closed = false
  private readonly listeners = new Map<string, Listener[]>()

  constructor(readonly url: string) {
    FakeEventSource.instances.push(this)
  }

  addEventListener(type: string, listener: Listener): void {
    const bucket = this.listeners.get(type) ?? []
    bucket.push(listener)
    this.listeners.set(type, bucket)
  }

  emit(type: string, event: unknown): void {
    for (const listener of this.listeners.get(type) ?? []) listener(event)
  }

  close(): void {
    this.closed = true
  }
}

const step = (over: Record<string, unknown> = {}) => ({
  id: '11111111-1111-1111-1111-111111111111',
  taskId: '22222222-2222-2222-2222-222222222222',
  title: 'Aggregate the rows',
  bodyMarkdown: null,
  state: 'RUNNING',
  done: false,
  runningAt: '2026-09-07T10:00:00Z',
  claimedAt: null,
  doneAt: null,
  position: 0,
  createdAt: '2026-09-07T09:00:00Z',
  updatedAt: '2026-09-07T10:00:00Z',
  ...over
})

describe('useStepStream', () => {
  beforeEach(() => {
    FakeEventSource.instances = []
    vi.stubGlobal('EventSource', FakeEventSource)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('opens one connection and hands a steps frame to the callback', () => {
    const onEvent = vi.fn<(taskId: TaskId, steps: unknown[]) => void>()
    const scope = effectScope()
    scope.run(() => useStepStream(onEvent))

    expect(FakeEventSource.instances).toHaveLength(1)
    expect(FakeEventSource.instances[0]!.url).toContain('/api/steps/stream')

    FakeEventSource.instances[0]!.emit('steps', {
      data: JSON.stringify({ taskId: step().taskId, steps: [step()] })
    })

    expect(onEvent).toHaveBeenCalledTimes(1)
    expect(onEvent.mock.calls[0]![0]).toBe(step().taskId)
    expect((onEvent.mock.calls[0]![1] as unknown[])[0]).toMatchObject({ state: 'RUNNING' })

    scope.stop()
  })

  it('drops a frame that does not parse instead of throwing', () => {
    const onEvent = vi.fn()
    const scope = effectScope()
    scope.run(() => useStepStream(onEvent))

    expect(() =>
      FakeEventSource.instances[0]!.emit('steps', { data: '{ not json' })
    ).not.toThrow()
    FakeEventSource.instances[0]!.emit('steps', { data: JSON.stringify({ taskId: 'x' }) })

    expect(onEvent).not.toHaveBeenCalled()
    scope.stop()
  })

  it('closes the connection when its scope is disposed', () => {
    const scope = effectScope()
    scope.run(() => useStepStream(vi.fn()))
    const source = FakeEventSource.instances[0]!

    scope.stop()

    expect(source.closed).toBe(true)
  })
})
