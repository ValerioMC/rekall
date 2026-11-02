import { beforeEach, describe, expect, it, vi } from 'vitest'
import { answerPermission, chatEventsUrl, interrupt, parseChatEvent, sendMessage } from '@/features/chat/api/chat.api'
import { openTerminal } from '@/features/claude/api/terminal.api'
import { TerminalSchema } from '@/features/claude/api/schemas/terminal.schema'
import type { TaskId, TerminalId } from '@/model/branded'

const client = vi.hoisted(() => vi.fn())

vi.mock('@/api/client', () => ({
  apiClient: client,
  request: <T>(operation: () => Promise<T>) => operation()
}))

const SESSION = 'aaaaaaaa-0000-4000-8000-000000000001' as TerminalId

describe('the chat api', () => {
  beforeEach(() => {
    client.mockReset()
    client.mockResolvedValue(undefined)
  })

  it('posts a message, an interrupt and a permission answer to the session', async () => {
    await sendMessage(SESSION, 'hello')
    await interrupt(SESSION)
    await answerPermission(SESSION, 'req/1', false)

    expect(client).toHaveBeenNthCalledWith(1, `/api/chat/${SESSION}/messages`, { method: 'POST', body: { text: 'hello' } })
    expect(client).toHaveBeenNthCalledWith(2, `/api/chat/${SESSION}/interrupt`, { method: 'POST' })
    expect(client).toHaveBeenNthCalledWith(3, `/api/chat/${SESSION}/permissions/req%2F1`, {
      method: 'POST',
      body: { allow: false }
    })
  })

  it('asks the stream for what came after the last event seen', () => {
    expect(chatEventsUrl(SESSION, 12)).toBe(`/api/chat/${SESSION}/events?after=12`)
  })

  it('reads the events the backend sends', () => {
    const edit = parseChatEvent(
      JSON.stringify({
        seq: 3,
        type: 'toolUse',
        toolUseId: 'u1',
        name: 'Edit',
        input: { tool: 'edit', filePath: 'a.rs', oldString: 'x', newString: 'y', replaceAll: false }
      })
    )
    const finished = parseChatEvent(
      JSON.stringify({
        seq: 4,
        type: 'turnFinished',
        usage: { inputTokens: 1, cacheCreationInputTokens: 2, cacheReadInputTokens: 3, outputTokens: 4 },
        durationMs: 900,
        outcome: 'completed'
      })
    )

    expect(edit).toMatchObject({ seq: 3, type: 'toolUse', input: { tool: 'edit', filePath: 'a.rs' } })
    expect(finished).toMatchObject({ type: 'turnFinished', outcome: 'completed' })
  })

  it('skips an event of a type it does not know, and one that is not JSON', () => {
    expect(parseChatEvent(JSON.stringify({ seq: 1, type: 'somethingNew' }))).toBeNull()
    expect(parseChatEvent('not json')).toBeNull()
  })
})

describe('the session kind on a terminal', () => {
  const view = {
    id: SESSION,
    projectId: 'bbbbbbbb-0000-4000-8000-000000000001',
    taskId: null,
    stepId: null,
    anchors: 'project:vega',
    workingDir: '/code/vega',
    projectLabel: 'vega',
    taskLabel: null,
    taskTitle: null,
    mode: 'GENERATE',
    skipPermissions: false,
    model: null,
    effort: null,
    live: true,
    startedAt: '',
    lastActivityAt: ''
  }

  it('is a terminal when the server says nothing', () => {
    const parsed = TerminalSchema.parse(view)

    expect(parsed.kind).toBe('TERMINAL')
    expect(parsed.awaitingPermission).toBe(false)
  })

  it('reads a chat session waiting for approval', () => {
    const parsed = TerminalSchema.parse({ ...view, kind: 'CHAT', awaitingPermission: true })

    expect(parsed.kind).toBe('CHAT')
    expect(parsed.awaitingPermission).toBe(true)
  })

  it('sends the kind it is asked to open, a terminal by default', async () => {
    client.mockResolvedValue(view)

    await openTerminal({ taskId: 't-1' as TaskId }, { skipPermissions: false })
    await openTerminal({ taskId: 't-1' as TaskId }, { skipPermissions: false, kind: 'CHAT' })

    expect(client.mock.calls[0]![1].body.kind).toBe('TERMINAL')
    expect(client.mock.calls[1]![1].body.kind).toBe('CHAT')
  })
})
