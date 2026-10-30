import { describe, expect, it } from 'vitest'
import { emptyTimeline, foldEvent } from '@/features/chat/lib/timeline'
import type { ChatBody, ChatEvent, ToolInput } from '@/features/chat/model/chat-event'
import type { AssistantItem, Timeline, ToolItem } from '@/features/chat/model/timeline'

const USAGE = { inputTokens: 10, cacheCreationInputTokens: 20, cacheReadInputTokens: 30, outputTokens: 40 }
const BASH: ToolInput = { tool: 'bash', command: 'ls', description: null }

function events(...bodies: ChatBody[]): ChatEvent[] {
  return bodies.map((body, index) => ({ ...body, seq: index + 1 }))
}

function fold(...bodies: ChatBody[]): Timeline {
  return events(...bodies).reduce(foldEvent, emptyTimeline())
}

function text(messageId: string, words: string, complete = false): ChatBody {
  return { type: 'assistantText', messageId, text: words, complete }
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
      { type: 'assistantThinking', messageId: 'm1', text: 'hm', complete: false },
      text('m1', 'Answer', true),
      { type: 'assistantThinking', messageId: 'm1', text: 'mm', complete: false }
    )

    expect(timeline.items.map((item) => item.kind)).toEqual(['thinking', 'assistant'])
    expect(timeline.items[0]).toMatchObject({ kind: 'thinking', text: 'hmmm' })
  })

  it('pairs a tool call with its result', () => {
    const timeline = fold(
      { type: 'toolUse', toolUseId: 'u1', name: 'Bash', input: BASH },
      { type: 'toolResult', toolUseId: 'u1', isError: false, excerpt: 'a.txt', truncated: false }
    )

    const tool = timeline.items[0] as ToolItem
    expect(timeline.items).toHaveLength(1)
    expect(tool.state).toBe('done')
    expect(tool.result).toEqual({ isError: false, excerpt: 'a.txt', truncated: false })
  })

  it('marks a tool running until its result and failed when the result is an error', () => {
    const running = fold({ type: 'toolUse', toolUseId: 'u1', name: 'Bash', input: BASH })
    const failed = fold(
      { type: 'toolUse', toolUseId: 'u1', name: 'Bash', input: BASH },
      { type: 'toolResult', toolUseId: 'u1', isError: true, excerpt: 'boom', truncated: false }
    )

    expect((running.items[0] as ToolItem).state).toBe('running')
    expect((failed.items[0] as ToolItem).state).toBe('failed')
  })

  it('ignores a result whose call it never saw', () => {
    const timeline = fold({ type: 'toolResult', toolUseId: 'ghost', isError: false, excerpt: '', truncated: false })

    expect(timeline.items).toEqual([])
  })

  it('tracks the single pending permission and the waiting state until it is resolved', () => {
    const waiting = fold(
      { type: 'userMessage', text: 'go' },
      { type: 'toolUse', toolUseId: 'u1', name: 'Bash', input: BASH },
      { type: 'permissionRequested', requestId: 'r1', toolUseId: 'u1', name: 'Bash', input: BASH }
    )
    const answered = fold(
      { type: 'userMessage', text: 'go' },
      { type: 'toolUse', toolUseId: 'u1', name: 'Bash', input: BASH },
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
      { type: 'toolUse', toolUseId: 'u1', name: 'Bash', input: BASH },
      { type: 'permissionRequested', requestId: 'r1', toolUseId: 'u1', name: 'Bash', input: BASH },
      { type: 'permissionResolved', requestId: 'r1', allowed: false },
      { type: 'toolResult', toolUseId: 'u1', isError: true, excerpt: 'denied', truncated: false }
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
      { type: 'turnFinished', usage: USAGE, durationMs: 1200, outcome: 'completed' },
      { type: 'userMessage', text: 'two' },
      { type: 'turnFinished', usage: USAGE, durationMs: 800, outcome: 'interrupted' }
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
      { type: 'turnFinished', usage: USAGE, durationMs: 1, outcome: 'completed' }
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

    foldEvent(before, { seq: 2, type: 'assistantText', messageId: 'm', text: 'x', complete: false })

    expect(JSON.stringify(before)).toBe(snapshot)
  })

  it('folds 5,000 events quickly', () => {
    const bodies: ChatBody[] = []
    for (let index = 0; index < 1250; index += 1) {
      bodies.push({ type: 'toolUse', toolUseId: `u${index}`, name: 'Bash', input: BASH })
      bodies.push({ type: 'toolResult', toolUseId: `u${index}`, isError: false, excerpt: 'ok', truncated: false })
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
})
