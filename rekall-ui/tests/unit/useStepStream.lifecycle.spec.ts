import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { effectScope } from 'vue'
import { useStepStream } from '@/composables/useStepStream'
import type { TaskId } from '@/model/branded'

/**
 * The live feed opens one `EventSource` and holds it for the life of the console. It is torn
 * down with the console's effect scope, and nothing about a checklist change reopens it. This
 * guards that: the connection a soak of the steps pane leaves behind is a leak of the whole
 * feed's closure, and a real one measured hundreds of megabytes for the editor before it was
 * fixed. See {@link AppMarkdownEditor.lifecycle.spec.ts} for the same discipline on that surface.
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
  readonly added: string[] = []
  private readonly listeners = new Map<string, Listener[]>()

  constructor(readonly url: string) {
    FakeEventSource.instances.push(this)
  }

  addEventListener(type: string, listener: Listener): void {
    this.added.push(type)
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

const frame = () => ({
  data: JSON.stringify({
    taskId: '22222222-2222-2222-2222-222222222222',
    steps: [
      {
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
        updatedAt: '2026-09-07T10:00:00Z'
      }
    ]
  })
})

describe('useStepStream lifecycle', () => {
  beforeEach(() => {
    FakeEventSource.instances = []
    vi.stubGlobal('EventSource', FakeEventSource)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('opens exactly one connection per scope and closes every one on dispose', () => {
    const scopes = Array.from({ length: 20 }, () => {
      const scope = effectScope()
      scope.run(() => useStepStream(vi.fn()))
      return scope
    })

    expect(FakeEventSource.instances).toHaveLength(20)

    scopes.forEach((scope) => scope.stop())

    expect(FakeEventSource.instances.every((source) => source.closed)).toBe(true)
  })

  it('registers each frame listener once, not once per checklist change', () => {
    const scope = effectScope()
    scope.run(() => useStepStream(vi.fn()))
    const source = FakeEventSource.instances[0]!

    // Many checklist changes arrive over the one connection.
    for (let i = 0; i < 50; i++) source.emit('steps', frame())

    expect(source.added.filter((type) => type === 'steps')).toHaveLength(1)
    expect(source.added.filter((type) => type === 'error')).toHaveLength(1)
    expect(source.added.filter((type) => type === 'open')).toHaveLength(1)

    scope.stop()
  })

  it('stops handing frames to the callback once the scope is disposed', () => {
    const onEvent = vi.fn<(taskId: TaskId, steps: unknown[]) => void>()
    const scope = effectScope()
    scope.run(() => useStepStream(onEvent))
    const source = FakeEventSource.instances[0]!

    source.emit('steps', frame())
    expect(onEvent).toHaveBeenCalledTimes(1)

    scope.stop()
    source.emit('steps', frame())

    expect(onEvent).toHaveBeenCalledTimes(1)
  })
})
