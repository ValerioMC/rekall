import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { isConsoleStreamOpen, subscribeToFrame } from '@/common/stream/consoleEventStream'

class FakeEventSource {
  static instances: FakeEventSource[] = []

  readyState = 0
  closed = false
  private readonly listeners = new Map<string, Array<(event: Event) => void>>()

  constructor(readonly url: string) {
    FakeEventSource.instances.push(this)
  }

  addEventListener(type: string, listener: (event: Event) => void): void {
    this.listeners.set(type, [...(this.listeners.get(type) ?? []), listener])
  }

  emit(type: string): void {
    for (const listener of this.listeners.get(type) ?? []) listener(new Event(type))
  }

  close(): void {
    this.closed = true
  }
}

describe('consoleEventStream', () => {
  beforeEach(() => {
    FakeEventSource.instances = []
    vi.stubGlobal('EventSource', FakeEventSource)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
  })

  it('serves every subscriber of every frame from one connection', () => {
    const unsubscribers = [
      subscribeToFrame('steps', vi.fn()),
      subscribeToFrame('steps', vi.fn()),
      subscribeToFrame('connection', vi.fn()),
      subscribeToFrame('diagram', vi.fn())
    ]

    expect(FakeEventSource.instances).toHaveLength(1)
    expect(FakeEventSource.instances[0]!.url).toContain('/api/steps/stream')

    unsubscribers.forEach((unsubscribe) => unsubscribe())
  })

  it('hands a frame to each subscriber of that frame only', () => {
    const first = vi.fn()
    const second = vi.fn()
    const other = vi.fn()
    const unsubscribers = [
      subscribeToFrame('steps', first),
      subscribeToFrame('steps', second),
      subscribeToFrame('diagram', other)
    ]

    FakeEventSource.instances[0]!.emit('steps')

    expect(first).toHaveBeenCalledTimes(1)
    expect(second).toHaveBeenCalledTimes(1)
    expect(other).not.toHaveBeenCalled()

    unsubscribers.forEach((unsubscribe) => unsubscribe())
  })

  it('closes the connection with the last subscriber and opens a new one for the next', () => {
    const unsubscribeFirst = subscribeToFrame('steps', vi.fn())
    const unsubscribeSecond = subscribeToFrame('diagram', vi.fn())

    unsubscribeFirst()
    expect(FakeEventSource.instances[0]!.closed).toBe(false)

    unsubscribeSecond()
    expect(FakeEventSource.instances[0]!.closed).toBe(true)

    const unsubscribeAgain = subscribeToFrame('steps', vi.fn())
    expect(FakeEventSource.instances).toHaveLength(2)
    unsubscribeAgain()
  })

  it('stops handing frames to a subscriber that left', () => {
    const stayer = vi.fn()
    const leaver = vi.fn()
    const unsubscribeStayer = subscribeToFrame('steps', stayer)
    const unsubscribeLeaver = subscribeToFrame('steps', leaver)

    unsubscribeLeaver()
    FakeEventSource.instances[0]!.emit('steps')

    expect(stayer).toHaveBeenCalledTimes(1)
    expect(leaver).not.toHaveBeenCalled()
    unsubscribeStayer()
  })

  it('reports the stream open only once the connection is established', () => {
    expect(isConsoleStreamOpen()).toBe(false)
    const unsubscribe = subscribeToFrame('steps', vi.fn())
    expect(isConsoleStreamOpen()).toBe(false)

    FakeEventSource.instances[0]!.readyState = 1
    expect(isConsoleStreamOpen()).toBe(true)

    unsubscribe()
  })
})
