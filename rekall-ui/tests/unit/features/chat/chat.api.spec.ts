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
        input: { tool: 'edit', filePath: 'a.rs', oldString: 'x', newString: 'y', replaceAll: false },
        parentToolUseId: null
      })
    )
    const finished = parseChatEvent(
      JSON.stringify({
        seq: 4,
        type: 'turnFinished',
        usage: { inputTokens: 1, cacheCreationInputTokens: 2, cacheReadInputTokens: 3, outputTokens: 4 },
        durationMs: 900,
        outcome: 'completed',
        costUsd: null,
        context: null
      })
    )

    expect(edit).toMatchObject({ seq: 3, type: 'toolUse', input: { tool: 'edit', filePath: 'a.rs' } })
    expect(finished).toMatchObject({ type: 'turnFinished', outcome: 'completed' })
  })

  it('reads each event the terminal parity adds', () => {
    const parse = (event: object) => parseChatEvent(JSON.stringify(event))
    const context = { usedTokens: 42000, windowTokens: 200000 }

    expect(
      parse({ seq: 1, type: 'sessionStarted', model: 'claude-opus', cwd: '/code', permissionMode: 'default', version: '2.1.331' })
    ).toMatchObject({ type: 'sessionStarted', permissionMode: 'default' })
    expect(parse({ seq: 2, type: 'commandOutput', command: '/context', text: 'Context Usage' })).toMatchObject({
      type: 'commandOutput',
      command: '/context'
    })
    expect(
      parse({
        seq: 3,
        type: 'toolUse',
        toolUseId: 'u1',
        name: 'TodoWrite',
        input: { tool: 'todos', items: [{ content: 'a', status: 'inProgress' }] },
        parentToolUseId: null
      })
    ).toMatchObject({ input: { tool: 'todos', items: [{ status: 'inProgress' }] } })
    expect(
      parse({
        seq: 4,
        type: 'toolUse',
        toolUseId: 'u2',
        name: 'Agent',
        input: { tool: 'agent', description: 'd', subagentType: null, prompt: 'p' },
        parentToolUseId: null
      })
    ).toMatchObject({ input: { tool: 'agent' } })
    expect(
      parse({
        seq: 5,
        type: 'toolUse',
        toolUseId: 'u3',
        name: 'WebSearch',
        input: { tool: 'web', name: 'WebSearch', target: 'q' },
        parentToolUseId: 'u2'
      })
    ).toMatchObject({ input: { tool: 'web' }, parentToolUseId: 'u2' })
    expect(
      parse({
        seq: 6,
        type: 'toolUse',
        toolUseId: 'u4',
        name: 'mcp__rekall__rekall_context',
        input: { tool: 'mcp', server: 'rekall', toolName: 'rekall_context', text: '{}' },
        parentToolUseId: null
      })
    ).toMatchObject({ input: { tool: 'mcp', toolName: 'rekall_context' } })
    expect(
      parse({
        seq: 7,
        type: 'toolResult',
        toolUseId: 'u1',
        isError: false,
        excerpt: '',
        truncated: false,
        summary: 'Read 120 lines',
        parentToolUseId: null
      })
    ).toMatchObject({ summary: 'Read 120 lines' })
    expect(
      parse({
        seq: 8,
        type: 'turnFinished',
        usage: { inputTokens: 1, cacheCreationInputTokens: 2, cacheReadInputTokens: 3, outputTokens: 4 },
        durationMs: 900,
        outcome: 'completed',
        costUsd: 0.0123,
        context
      })
    ).toMatchObject({ costUsd: 0.0123, context })
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
    taskCode: null,
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

  it('reads a Confluence connection attached to the terminal', () => {
    const connection = { connectionId: 'cccccccc-0000-4000-8000-000000000001', label: 'wiki', kind: 'CONFLUENCE', credentialSet: true }

    const parsed = TerminalSchema.parse({ ...view, connections: [connection] })

    expect(parsed.connections[0]!.kind).toBe('CONFLUENCE')
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
