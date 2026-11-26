import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { effectScope, type EffectScope } from 'vue'
import { flushPromises } from '@vue/test-utils'
import { useFollowSelectedChat } from '@/features/chat/composables/useFollowSelectedChat'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useConsoleStore } from '@/stores/console.store'
import type { TaskId, TerminalId } from '@/model/branded'
import { CHAT_ID, FakeEventSource, chatSession } from './support'
import { installStorage } from '../../support/memoryStorage'

vi.mock('@/features/claude/api/terminal.api', () => ({
  fetchTerminals: vi.fn().mockResolvedValue([]),
  openTerminal: vi.fn(),
  closeTerminal: vi.fn(),
  sendTerminalInput: vi.fn()
}))

const OWN = 't-1' as TaskId
const OTHER = 't-2' as TaskId
const OTHER_CHAT = 'chat-2' as TerminalId

describe('following the selected task\'s chat', () => {
  let scope: EffectScope

  async function follow(): Promise<void> {
    scope = effectScope()
    scope.run(() => useFollowSelectedChat())
    await flushPromises()
  }

  async function select(taskId: TaskId): Promise<void> {
    useConsoleStore().selectedTaskId = taskId
    await flushPromises()
  }

  function openStreams(): FakeEventSource[] {
    return FakeEventSource.instances.filter((source) => !source.closed)
  }

  beforeEach(() => {
    setActivePinia(createPinia())
    installStorage()
    FakeEventSource.instances = []
    vi.stubGlobal('EventSource', FakeEventSource)
    document.title = 'Rekall'
  })

  afterEach(() => {
    scope.stop()
    vi.unstubAllGlobals()
  })

  it('holds the stream of the selected task\'s chat, and moves when the selection changes', async () => {
    useTerminalStore().upsert(chatSession())
    useTerminalStore().upsert(chatSession({ id: OTHER_CHAT, taskId: OTHER }))
    useConsoleStore().selectedTaskId = OWN
    await follow()
    expect(openStreams().map((source) => source.url)).toEqual([`/api/chat/${CHAT_ID}/events?after=0`])

    await select(OTHER)

    expect(useChatStore().activeChatId).toBe(OTHER_CHAT)
    expect(openStreams().map((source) => source.url)).toEqual([`/api/chat/${OTHER_CHAT}/events?after=0`])
  })

  it('picks up a chat that opens on the selected task', async () => {
    await select(OWN)
    await follow()
    expect(openStreams()).toHaveLength(0)

    useTerminalStore().upsert(chatSession())
    await flushPromises()

    expect(useChatStore().activeChatId).toBe(CHAT_ID)
    expect(openStreams()).toHaveLength(1)
  })

  it('stays on the task\'s ended chat until a live one opens on it', async () => {
    useTerminalStore().upsert(chatSession())
    await select(OWN)
    await follow()

    useTerminalStore().markEnded(CHAT_ID)
    await flushPromises()
    expect(useChatStore().activeChatId).toBe(CHAT_ID)

    useTerminalStore().upsert(chatSession({ id: OTHER_CHAT }))
    await flushPromises()
    expect(useChatStore().activeChatId).toBe(OTHER_CHAT)
  })

  it('lets go of another task\'s chat on a task with none', async () => {
    useTerminalStore().upsert(chatSession())
    await select(OWN)
    await follow()

    await select(OTHER)

    expect(useChatStore().activeChatId).toBeNull()
    expect(openStreams()).toHaveLength(0)
  })

  it('keeps a chat on no task until the selected task has its own', async () => {
    useTerminalStore().upsert(chatSession({ taskId: null }))
    useChatStore().select(CHAT_ID)
    await follow()

    await select(OWN)
    expect(useChatStore().activeChatId).toBe(CHAT_ID)

    useTerminalStore().upsert(chatSession({ id: OTHER_CHAT }))
    await flushPromises()
    expect(useChatStore().activeChatId).toBe(OTHER_CHAT)
  })

  it('hears an approval with no chat on screen, and marks the browser tab title until it is answered', async () => {
    useTerminalStore().upsert(chatSession())
    await select(OWN)
    await follow()
    const stream = openStreams()[0]!
    stream.open()

    stream.emit({
      seq: 1,
      type: 'permissionRequested',
      requestId: 'req-1',
      toolUseId: 'u1',
      name: 'Bash',
      input: { tool: 'bash', command: 'ls', description: null }
    })
    await flushPromises()
    expect(useTerminalStore().terminals[0]?.awaitingPermission).toBe(true)
    expect(document.title).toBe('• Rekall')

    stream.emit({ seq: 2, type: 'permissionResolved', requestId: 'req-1', allowed: true })
    await flushPromises()
    expect(document.title).toBe('Rekall')
  })

  it('tells the chat store how the stream is connected', async () => {
    useTerminalStore().upsert(chatSession())
    await select(OWN)
    await follow()
    const stream = openStreams()[0]!

    stream.open()
    await flushPromises()
    expect(useChatStore().connection).toEqual({ connected: true, reconnecting: false })

    stream.drop()
    await flushPromises()
    expect(useChatStore().connection).toEqual({ connected: false, reconnecting: true })
  })
})
