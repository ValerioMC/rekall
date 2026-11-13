import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useChatStore } from '@/features/chat/store/chat.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import type { ChatEvent } from '@/features/chat/model/chat-event'
import type { Terminal } from '@/features/claude/model/terminal'
import type { ProjectId, TaskId, TaskStepId, TerminalId } from '@/model/branded'

const chatApi = { sendMessage: vi.fn(), interrupt: vi.fn(), answerPermission: vi.fn() }
const terminalApi = { fetchTerminals: vi.fn() }

vi.mock('@/features/chat/api/chat.api', () => ({
  sendMessage: (...a: unknown[]) => chatApi.sendMessage(...a),
  interrupt: (...a: unknown[]) => chatApi.interrupt(...a),
  answerPermission: (...a: unknown[]) => chatApi.answerPermission(...a)
}))

vi.mock('@/features/claude/api/terminal.api', () => ({
  fetchTerminals: (...a: unknown[]) => terminalApi.fetchTerminals(...a),
  openTerminal: vi.fn(),
  closeTerminal: vi.fn(),
  sendTerminalInput: vi.fn()
}))

const TASK = 't-1' as TaskId
const CHAT = 'chat-1' as TerminalId

function session(over: Partial<Terminal> = {}): Terminal {
  return {
    id: CHAT,
    projectId: 'p-1' as ProjectId,
    taskId: TASK,
    stepId: null as TaskStepId | null,
    anchors: 'project:vega task:report-builder',
    workingDir: '/code/vega',
    projectLabel: 'vega',
    taskCode: 'report-builder',
    taskTitle: 'Report builder',
    mode: 'WORK',
    skipPermissions: false,
    model: null,
    effort: null,
    kind: 'CHAT',
    live: true,
    awaitingPermission: false,
    startedAt: '',
    lastActivityAt: '',
    connections: [],
    ...over
  }
}

const bash = { tool: 'bash', command: 'ls', description: null } as const

function permissionRequested(seq: number): ChatEvent {
  return { seq, type: 'permissionRequested', requestId: 'req-1', toolUseId: 'u1', name: 'Bash', input: bash }
}

describe('the chat store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    Object.values(chatApi).forEach((mock) => mock.mockReset().mockResolvedValue(undefined))
    terminalApi.fetchTerminals.mockReset()
  })

  it('keeps one timeline per session as their events arrive', () => {
    const store = useChatStore()

    store.ingest(CHAT, { seq: 1, type: 'userMessage', text: 'go' })
    store.ingest('chat-2' as TerminalId, { seq: 1, type: 'userMessage', text: 'other' })
    store.ingest(CHAT, { seq: 2, type: 'assistantText', messageId: 'm1', text: 'Hi', complete: true })

    expect(store.timelineOf(CHAT).items).toHaveLength(2)
    expect(store.timelineOf('chat-2' as TerminalId).items).toHaveLength(1)
    expect(store.timelineOf('nobody' as TerminalId).items).toEqual([])
  })

  it('does not fold an event twice', () => {
    const store = useChatStore()
    const event: ChatEvent = { seq: 1, type: 'userMessage', text: 'go' }

    store.ingest(CHAT, event)
    const first = store.timelineOf(CHAT)
    store.ingest(CHAT, event)

    expect(store.timelineOf(CHAT)).toBe(first)
  })

  it('shows the timeline of the selected session', () => {
    const store = useChatStore()
    store.ingest(CHAT, { seq: 1, type: 'userMessage', text: 'go' })

    store.select(CHAT)

    expect(store.activeTimeline.items).toHaveLength(1)
    store.select(null)
    expect(store.activeTimeline.items).toEqual([])
  })

  it('sends a message, and says so while it is on its way', async () => {
    let finish: () => void = () => undefined
    chatApi.sendMessage.mockReturnValue(new Promise<void>((resolve) => (finish = resolve)))
    const store = useChatStore()

    const sending = store.send(CHAT, 'hello')

    expect(chatApi.sendMessage).toHaveBeenCalledWith(CHAT, 'hello')
    expect(store.sending).toBe(true)
    finish()
    await sending
    expect(store.sending).toBe(false)
  })

  it('lets a failed send reach the caller and stops saying it is sending', async () => {
    chatApi.sendMessage.mockRejectedValue(new Error('refused'))
    const store = useChatStore()

    await expect(store.send(CHAT, 'hello')).rejects.toThrow('refused')

    expect(store.sending).toBe(false)
  })

  it('interrupts the session', async () => {
    const store = useChatStore()

    await store.stop(CHAT)

    expect(chatApi.interrupt).toHaveBeenCalledWith(CHAT)
  })

  it('answers the pending permission by its request id', async () => {
    const store = useChatStore()
    store.ingest(CHAT, permissionRequested(1))

    await store.allow(CHAT)
    await store.deny(CHAT)

    expect(chatApi.answerPermission).toHaveBeenNthCalledWith(1, CHAT, 'req-1', true)
    expect(chatApi.answerPermission).toHaveBeenNthCalledWith(2, CHAT, 'req-1', false)
  })

  it('clears the session record awaiting a permission once the answer is folded', () => {
    const terminals = useTerminalStore()
    terminals.upsert(session({ awaitingPermission: true }))
    const store = useChatStore()
    store.ingest(CHAT, permissionRequested(1))

    store.ingest(CHAT, { seq: 2, type: 'permissionResolved', requestId: 'req-1', allowed: false })

    expect(terminals.terminals.find((terminal) => terminal.id === CHAT)?.awaitingPermission).toBe(false)
  })

  it('marks the session record awaiting a permission when one is requested', () => {
    const terminals = useTerminalStore()
    terminals.upsert(session())
    const store = useChatStore()

    store.ingest(CHAT, permissionRequested(1))

    expect(terminals.terminals.find((terminal) => terminal.id === CHAT)?.awaitingPermission).toBe(true)
  })

  it('answers nothing when no permission is pending', async () => {
    const store = useChatStore()

    await store.allow(CHAT)

    expect(chatApi.answerPermission).not.toHaveBeenCalled()
  })

  describe('when a session ends', () => {
    beforeEach(() => vi.useFakeTimers())

    async function end(store: ReturnType<typeof useChatStore>): Promise<void> {
      store.ingest(CHAT, { seq: 9, type: 'ended', exitCode: 0, detail: 'done' })
      await vi.runAllTimersAsync()
    }

    it('marks it ended and moves the window to the session opened next on the task', async () => {
      terminalApi.fetchTerminals.mockResolvedValue([session({ id: 'chat-2' as TerminalId })])
      const terminals = useTerminalStore()
      terminals.terminals = [session()]
      const store = useChatStore()
      store.select(CHAT)

      await end(store)

      expect(terminals.terminals.find((terminal) => terminal.id === CHAT)?.live).toBe(false)
      expect(store.activeChatId).toBe('chat-2')
      expect(store.isFollowing(CHAT)).toBe(false)
      vi.useRealTimers()
    })

    it('hands the pane to a terminal when the next session is one', async () => {
      terminalApi.fetchTerminals.mockResolvedValue([session({ id: 'term-2' as TerminalId, kind: 'TERMINAL' })])
      const terminals = useTerminalStore()
      terminals.terminals = [session()]
      const store = useChatStore()
      store.select(CHAT)

      await end(store)

      expect(store.activeChatId).toBeNull()
      expect(terminals.activeTerminalId).toBe('term-2')
      vi.useRealTimers()
    })

    it('keeps the window where it is when the person moved to another session meanwhile', async () => {
      terminalApi.fetchTerminals.mockResolvedValue([session({ id: 'chat-2' as TerminalId })])
      const terminals = useTerminalStore()
      terminals.terminals = [session()]
      const store = useChatStore()
      store.select('other' as TerminalId)

      await end(store)

      expect(store.activeChatId).toBe('other')
      vi.useRealTimers()
    })

    it('looks for no next session when it was already over, as when an old one is opened again', async () => {
      const terminals = useTerminalStore()
      terminals.terminals = [session({ live: false })]
      const store = useChatStore()

      await end(store)

      expect(terminalApi.fetchTerminals).not.toHaveBeenCalled()
      vi.useRealTimers()
    })

    it('says it is waiting for the next session while it looks', async () => {
      terminalApi.fetchTerminals.mockResolvedValue([])
      const terminals = useTerminalStore()
      terminals.terminals = [session()]
      const store = useChatStore()

      store.ingest(CHAT, { seq: 9, type: 'ended', exitCode: 0, detail: 'done' })
      await vi.advanceTimersByTimeAsync(100)

      expect(store.isFollowing(CHAT)).toBe(true)
      await vi.runAllTimersAsync()
      expect(store.isFollowing(CHAT)).toBe(false)
      vi.useRealTimers()
    })
  })
})
