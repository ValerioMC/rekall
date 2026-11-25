import { describe, expect, it } from 'vitest'
import { emptyTimeline, foldEvent } from '@/features/chat/lib/timeline'
import type { ChatBody, ChatEvent, ToolInput } from '@/features/chat/model/chat-event'
import type { AssistantItem, CommandOutputItem, SessionItem, Timeline, ToolItem } from '@/features/chat/model/timeline'

const USAGE = { inputTokens: 10, cacheCreationInputTokens: 20, cacheReadInputTokens: 30, outputTokens: 40 }
const BASH: ToolInput = { tool: 'bash', command: 'ls', description: null }

function events(...bodies: ChatBody[]): ChatEvent[] {
  return bodies.map((body, index) => ({ ...body, seq: index + 1 }))
}

function fold(...bodies: ChatBody[]): Timeline {
  return events(...bodies).reduce(foldEvent, emptyTimeline())
}

function text(messageId: string, words: string, complete = false, parentToolUseId: string | null = null): ChatBody {
  return { type: 'assistantText', messageId, text: words, complete, parentToolUseId }
}

function toolUse(toolUseId: string, input: ToolInput = BASH, parentToolUseId: string | null = null): ChatBody {
  return { type: 'toolUse', toolUseId, name: 'Tool', input, parentToolUseId }
}

function toolResult(
  toolUseId: string,
  over: { isError?: boolean; excerpt?: string; summary?: string | null } = {}
): ChatBody {
  return {
    type: 'toolResult',
    toolUseId,
    isError: over.isError ?? false,
    excerpt: over.excerpt ?? '',
    truncated: false,
    summary: over.summary ?? null,
    parentToolUseId: null
  }
}

function turnFinished(over: Partial<Extract<ChatBody, { type: 'turnFinished' }>> = {}): ChatBody {
  return {
    type: 'turnFinished',
    usage: USAGE,
    durationMs: 1,
    outcome: 'completed',
    costUsd: null,
    context: null,
    ...over
  }
}

describe('the chat timeline reducer', () => {
  it('merges the partial pieces of an assistant message and keeps the complete text', () => {
    const streaming = fold(text('m1', 'Hel'), text('m1', 'lo'))
    const finished = fold(text('m1', 'Hel'), text('m1', 'lo'), text('m1', 'Hello there', true))

    expect((streaming.items[0] as AssistantItem).text).toBe('Hello')
    expect((streaming.items[0] as AssistantItem).complete).toBe(false)
    expect(finished.items).toHaveLength(1)
    expect((finished.items[0] as AssistantItem).text).toBe('Hello there')
    expect((finished.items[0] as AssistantItem).complete).toBe(true)
  })

  it('ignores a partial that arrives after the complete text of its message', () => {
    const timeline = fold(text('m1', 'Hello', true), text('m1', ' straggler'))

    expect((timeline.items[0] as AssistantItem).text).toBe('Hello')
  })

  it('shows a complete text that came with no partials before it', () => {
    const timeline = fold(text('m1', 'All at once', true))

    expect(timeline.items).toHaveLength(1)
    expect((timeline.items[0] as AssistantItem).complete).toBe(true)
  })

  it('keeps the thinking and the text of one message apart', () => {
    const timeline = fold(
      { type: 'assistantThinking', messageId: 'm1', text: 'hm', complete: false, parentToolUseId: null },
      text('m1', 'Answer', true),
      { type: 'assistantThinking', messageId: 'm1', text: 'mm', complete: false, parentToolUseId: null }
    )

    expect(timeline.items.map((item) => item.kind)).toEqual(['thinking', 'assistant'])
    expect(timeline.items[0]).toMatchObject({ kind: 'thinking', text: 'hmmm' })
  })

  it('pairs a tool call with its result', () => {
    const timeline = fold(
      toolUse('u1'),
      toolResult('u1', { excerpt: 'a.txt' })
    )

    const tool = timeline.items[0] as ToolItem
    expect(timeline.items).toHaveLength(1)
    expect(tool.state).toBe('done')
    expect(tool.result).toEqual({ isError: false, excerpt: 'a.txt', truncated: false, summary: null })
  })

  it('marks a tool running until its result and failed when the result is an error', () => {
    const running = fold(toolUse('u1'))
    const failed = fold(
      toolUse('u1'),
      toolResult('u1', { isError: true, excerpt: 'boom' })
    )

    expect((running.items[0] as ToolItem).state).toBe('running')
    expect((failed.items[0] as ToolItem).state).toBe('failed')
  })

  it('ignores a result whose call it never saw', () => {
    const timeline = fold(toolResult('ghost'))

    expect(timeline.items).toEqual([])
  })

  it('tracks the single pending permission and the waiting state until it is resolved', () => {
    const waiting = fold(
      { type: 'userMessage', text: 'go' },
      toolUse('u1'),
      { type: 'permissionRequested', requestId: 'r1', toolUseId: 'u1', name: 'Bash', input: BASH }
    )
    const answered = fold(
      { type: 'userMessage', text: 'go' },
      toolUse('u1'),
      { type: 'permissionRequested', requestId: 'r1', toolUseId: 'u1', name: 'Bash', input: BASH },
      { type: 'permissionResolved', requestId: 'r1', allowed: true }
    )

    expect(waiting.turn).toBe('waiting')
    expect(waiting.pending).toMatchObject({ requestId: 'r1', toolUseId: 'u1', name: 'Bash' })
    expect(answered.turn).toBe('working')
    expect(answered.pending).toBeNull()
    expect((answered.items[1] as ToolItem).state).toBe('running')
  })

  it('shows a refused call as denied even though its result is an error', () => {
    const timeline = fold(
      toolUse('u1'),
      { type: 'permissionRequested', requestId: 'r1', toolUseId: 'u1', name: 'Bash', input: BASH },
      { type: 'permissionResolved', requestId: 'r1', allowed: false },
      toolResult('u1', { isError: true, excerpt: 'denied' })
    )

    expect((timeline.items[0] as ToolItem).state).toBe('denied')
  })

  it('keeps the pending permission when an older request is resolved', () => {
    const timeline = fold(
      { type: 'permissionRequested', requestId: 'r2', toolUseId: 'u2', name: 'Bash', input: BASH },
      { type: 'permissionResolved', requestId: 'r1', allowed: true }
    )

    expect(timeline.pending?.requestId).toBe('r2')
    expect(timeline.turn).toBe('waiting')
  })

  it('totals the tokens of finished turns and goes idle with a footer', () => {
    const timeline = fold(
      { type: 'userMessage', text: 'one' },
      turnFinished({ durationMs: 1200 }),
      { type: 'userMessage', text: 'two' },
      turnFinished({ durationMs: 800, outcome: 'interrupted' })
    )

    expect(timeline.tokens).toEqual({
      inputTokens: 20,
      cacheCreationInputTokens: 40,
      cacheReadInputTokens: 60,
      outputTokens: 80
    })
    expect(timeline.turn).toBe('idle')
    expect(timeline.items.filter((item) => item.kind === 'turn-footer')).toHaveLength(2)
  })

  it('is working from the first message of a turn', () => {
    expect(fold({ type: 'userMessage', text: 'go' }).turn).toBe('working')
    expect(emptyTimeline().turn).toBe('idle')
  })

  it('ends the session and drops any pending permission', () => {
    const timeline = fold(
      { type: 'permissionRequested', requestId: 'r1', toolUseId: 'u1', name: 'Bash', input: BASH },
      { type: 'ended', exitCode: 1, detail: 'killed' }
    )

    expect(timeline.ended).toBe(true)
    expect(timeline.pending).toBeNull()
    expect(timeline.turn).toBe('idle')
    expect(timeline.items.at(-1)).toMatchObject({ kind: 'ended', exitCode: 1, detail: 'killed' })
  })

  it('shows notices in place', () => {
    const timeline = fold({ type: 'notice', kind: 'compaction', text: 'Context compacted' })

    expect(timeline.items[0]).toMatchObject({ kind: 'notice', noticeKind: 'compaction' })
  })

  it('folds a repeated seq after a reconnect once', () => {
    const replay = events(
      { type: 'userMessage', text: 'go' },
      text('m1', 'Hi', true),
      turnFinished()
    )
    const once = replay.reduce(foldEvent, emptyTimeline())
    const twice = replay.reduce(foldEvent, once)

    expect(twice).toBe(once)
    expect(twice.tokens.outputTokens).toBe(40)
    expect(twice.items).toHaveLength(3)
  })

  it('ignores an event type it does not know', () => {
    const unknown = { seq: 1, type: 'somethingNew', payload: 1 } as unknown as ChatEvent

    const timeline = foldEvent(emptyTimeline(), unknown)

    expect(timeline.items).toEqual([])
    expect(timeline.lastSeq).toBe(1)
  })

  it('does not change the timeline it was given', () => {
    const before = fold({ type: 'userMessage', text: 'go' })
    const snapshot = JSON.stringify(before)

    foldEvent(before, { seq: 2, ...text('m', 'x') } as ChatEvent)

    expect(JSON.stringify(before)).toBe(snapshot)
  })

  it('folds 5,000 events quickly', () => {
    const bodies: ChatBody[] = []
    for (let index = 0; index < 1250; index += 1) {
      bodies.push(toolUse(`u${index}`))
      bodies.push(toolResult(`u${index}`, { excerpt: 'ok' }))
      bodies.push(text(`m${index}`, 'partial '))
      bodies.push(text(`m${index}`, 'partial text', true))
    }
    const stream = events(...bodies)

    const started = performance.now()
    const timeline = stream.reduce(foldEvent, emptyTimeline())
    const elapsed = performance.now() - started

    expect(timeline.items).toHaveLength(2500)
    expect(timeline.lastSeq).toBe(5000)
    expect(elapsed).toBeLessThan(1000)
  })

  it('shows the session header and the output of a local command in place', () => {
    const timeline = fold(
      { type: 'sessionStarted', model: 'claude-opus', cwd: '/code/vega', permissionMode: 'default', version: '2.1.331' },
      { type: 'commandOutput', command: '/context', text: 'Context Usage\n10%' }
    )

    expect(timeline.items[0]).toEqual<SessionItem>({
      kind: 'session',
      key: 'session-1',
      model: 'claude-opus',
      cwd: '/code/vega',
      permissionMode: 'default',
      version: '2.1.331'
    })
    expect(timeline.items[1]).toEqual<CommandOutputItem>({
      kind: 'command-output',
      key: 'command-2',
      command: '/context',
      text: 'Context Usage\n10%'
    })
    expect(timeline.turn).toBe('idle')
  })

  it('puts the summary of a result on its tool', () => {
    const timeline = fold(toolUse('u1'), toolResult('u1', { summary: 'exit code 1', isError: true }))

    expect((timeline.items[0] as ToolItem).result?.summary).toBe('exit code 1')
  })

  it('keeps the items of a subagent in order, each carrying the Agent call it belongs to', () => {
    const agent: ToolInput = { tool: 'agent', description: 'Find it', subagentType: null, prompt: 'go' }

    const timeline = fold(
      toolUse('agent-1', agent),
      text('m1', 'inner', true, 'agent-1'),
      toolUse('u2', BASH, 'agent-1'),
      text('m2', 'outer', true)
    )

    expect(timeline.items.map((item) => item.key)).toEqual(['tool-agent-1', 'assistant-m1', 'tool-u2', 'assistant-m2'])
    expect(timeline.items.map((item) => (item as { parentToolUseId: string | null }).parentToolUseId)).toEqual([
      null,
      'agent-1',
      'agent-1',
      null
    ])
  })

  it('adds the cost of each turn to a total and keeps the latest context fill', () => {
    const first = { usedTokens: 1000, windowTokens: 200000 }
    const second = { usedTokens: 3000, windowTokens: 200000 }

    const timeline = fold(
      turnFinished({ costUsd: 0.25, context: first }),
      turnFinished({ costUsd: 0.5, context: second }),
      turnFinished({ costUsd: null, context: null })
    )

    expect(timeline.cost).toBeCloseTo(0.75)
    expect(timeline.context).toEqual(second)
    expect(timeline.items.filter((item) => item.kind === 'turn-footer')[0]).toMatchObject({ costUsd: 0.25, context: first })
  })

  it('has no cost and no context before a turn finishes', () => {
    expect(emptyTimeline()).toMatchObject({ cost: 0, context: null })
  })
})
