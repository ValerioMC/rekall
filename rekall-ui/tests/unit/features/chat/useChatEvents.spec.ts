import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { effectScope, nextTick, ref, type EffectScope, type Ref } from 'vue'
import { useChatEvents, type ChatEventStream } from '@/features/chat/composables/useChatEvents'
import { useChatStore } from '@/features/chat/store/chat.store'
import type { TerminalId } from '@/model/branded'

vi.mock('@/features/claude/api/terminal.api', () => ({
  fetchTerminals: vi.fn().mockResolvedValue([]),
  openTerminal: vi.fn(),
  closeTerminal: vi.fn(),
  sendTerminalInput: vi.fn()
}))

class FakeEventSource {
  static readonly CONNECTING = 0
  static readonly OPEN = 1
  static readonly CLOSED = 2
  static instances: FakeEventSource[] = []

  readyState = FakeEventSource.CONNECTING
  closed = false
  onopen: (() => void) | null = null
  onmessage: ((message: MessageEvent<string>) => void) | null = null
  onerror: (() => void) | null = null

  constructor(readonly url: string) {
    FakeEventSource.instances.push(this)
  }

  close(): void {
    this.closed = true
    this.readyState = FakeEventSource.CLOSED
  }

  open(): void {
    this.readyState = FakeEventSource.OPEN
    this.onopen?.()
  }

  emit(payload: unknown): void {
    this.onmessage?.({ data: typeof payload === 'string' ? payload : JSON.stringify(payload) } as MessageEvent<string>)
  }

  drop(): void {
    this.readyState = FakeEventSource.CONNECTING
    this.onerror?.()
  }
}

const FIRST = 'chat-1' as TerminalId
const SECOND = 'chat-2' as TerminalId

describe('useChatEvents', () => {
  let scope: EffectScope
  let session: Ref<TerminalId | null>
  let stream: ChatEventStream

  function start(initial: TerminalId | null): void {
    session = ref<TerminalId | null>(initial)
    scope = effectScope()
    stream = scope.run(() => useChatEvents(session))!
  }

  function latest(): FakeEventSource {
    return FakeEventSource.instances.at(-1)!
  }

  beforeEach(() => {
    setActivePinia(createPinia())
    FakeEventSource.instances = []
    vi.stubGlobal('EventSource', FakeEventSource)
  })

  afterEach(() => {
    scope.stop()
    vi.unstubAllGlobals()
  })

  it('opens nothing without a session', () => {
    start(null)

    expect(FakeEventSource.instances).toHaveLength(0)
    expect(stream.connected.value).toBe(false)
  })

  it('opens the session stream from the start, and is connected once it opens', () => {
    start(FIRST)

    expect(latest().url).toBe(`/api/chat/${FIRST}/events?after=0`)
    expect(stream.connected.value).toBe(false)
    latest().open()
    expect(stream.connected.value).toBe(true)
  })

  it('puts what arrives into the session timeline', () => {
    start(FIRST)

    latest().emit({ seq: 1, type: 'userMessage', text: 'go' })
    latest().emit({ seq: 2, type: 'somethingNew' })
    latest().emit('not json')

    const chat = useChatStore()
    expect(chat.timelineOf(FIRST).items).toHaveLength(1)
    expect(chat.timelineOf(FIRST).lastSeq).toBe(1)
  })

  it('says it is reconnecting while the browser retries, and not once it is back', () => {
    start(FIRST)
    latest().open()

    latest().drop()
    expect(stream.connected.value).toBe(false)
    expect(stream.reconnecting.value).toBe(true)

    latest().open()
    expect(stream.connected.value).toBe(true)
    expect(stream.reconnecting.value).toBe(false)
  })

  it('does not say it is reconnecting when the server refused the stream', () => {
    start(FIRST)

    latest().readyState = FakeEventSource.CLOSED
    latest().onerror?.()

    expect(stream.reconnecting.value).toBe(false)
  })

  it('asks a later connection only for what came after the last event it has', async () => {
    start(FIRST)
    latest().emit({ seq: 1, type: 'userMessage', text: 'go' })
    latest().emit({ seq: 2, type: 'assistantText', messageId: 'm', text: 'Hi', complete: true })

    session.value = null
    await nextTick()
    session.value = FIRST
    await nextTick()

    expect(latest().url).toBe(`/api/chat/${FIRST}/events?after=2`)
  })

  it('closes the previous stream when the session changes', async () => {
    start(FIRST)
    const first = latest()
    first.open()

    session.value = SECOND
    await nextTick()

    expect(first.closed).toBe(true)
    expect(latest().url).toContain(SECOND)
    expect(FakeEventSource.instances.filter((source) => !source.closed)).toHaveLength(1)
    expect(stream.connected.value).toBe(false)
  })

  it('closes its stream when the scope ends', () => {
    start(FIRST)

    scope.stop()

    expect(latest().closed).toBe(true)
  })

  it('closes the stream once the session ended, and does not open an ended one again', async () => {
    start(FIRST)
    latest().open()

    latest().emit({ seq: 1, type: 'ended', exitCode: 0, detail: 'done' })

    expect(latest().closed).toBe(true)
    expect(stream.ended.value).toBe(true)
    expect(stream.connected.value).toBe(false)

    session.value = null
    await nextTick()
    session.value = FIRST
    await nextTick()
    expect(FakeEventSource.instances).toHaveLength(1)
  })
})
