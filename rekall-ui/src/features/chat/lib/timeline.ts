import type { ChatBody, ChatEvent, TurnUsage } from '../model/chat-event'
import type {
  AssistantItem,
  ThinkingItem,
  Timeline,
  TimelineItem,
  ToolItem
} from '../model/timeline'

const NO_TOKENS: TurnUsage = {
  inputTokens: 0,
  cacheCreationInputTokens: 0,
  cacheReadInputTokens: 0,
  outputTokens: 0
}

export function emptyTimeline(): Timeline {
  return { items: [], lastSeq: 0, turn: 'idle', pending: null, tokens: NO_TOKENS, ended: false }
}

type Draft = { -readonly [Field in keyof Timeline]: Timeline[Field] } & { items: TimelineItem[] }

/** The timeline with `event` folded in; an event at or before `lastSeq` was already seen and changes nothing. */
export function foldEvent(timeline: Timeline, event: ChatEvent): Timeline {
  if (event.seq <= timeline.lastSeq) return timeline
  const draft = draftOf(timeline)
  apply(draft, event)
  return draft
}

function draftOf(timeline: Timeline): Draft {
  return { ...timeline, items: timeline.items.slice() }
}

function apply(draft: Draft, event: ChatEvent): void {
  draft.lastSeq = event.seq
  const body: ChatBody = event
  switch (body.type) {
    case 'userMessage':
      draft.items.push({ kind: 'user', key: `user-${event.seq}`, text: body.text })
      draft.turn = 'working'
      break
    case 'assistantText':
      foldAssistantText(draft, body)
      break
    case 'assistantThinking':
      foldThinking(draft, body)
      break
    case 'toolUse':
      foldToolUse(draft, body)
      break
    case 'toolResult':
      foldToolResult(draft, body)
      break
    case 'permissionRequested':
      draft.pending = {
        requestId: body.requestId,
        toolUseId: body.toolUseId,
        name: body.name,
        input: body.input
      }
      draft.turn = 'waiting'
      break
    case 'permissionResolved':
      foldPermissionResolved(draft, body)
      break
    case 'turnFinished':
      foldTurnFinished(draft, event.seq, body)
      break
    case 'notice':
      draft.items.push({
        kind: 'notice',
        key: `notice-${event.seq}`,
        noticeKind: body.kind,
        text: body.text
      })
      break
    case 'ended':
      draft.items.push({
        kind: 'ended',
        key: `ended-${event.seq}`,
        exitCode: body.exitCode,
        detail: body.detail
      })
      draft.ended = true
      draft.turn = 'idle'
      draft.pending = null
      break
    default:
      break
  }
}

function lastIndexWhere(items: readonly TimelineItem[], matches: (item: TimelineItem) => boolean): number {
  for (let index = items.length - 1; index >= 0; index -= 1) {
    if (matches(items[index]!)) return index
  }
  return -1
}

function foldAssistantText(
  draft: Draft,
  body: Extract<ChatBody, { type: 'assistantText' }>
): void {
  const index = lastIndexWhere(
    draft.items,
    (item) => item.kind === 'assistant' && item.messageId === body.messageId
  )
  if (index < 0) {
    draft.items.push({
      kind: 'assistant',
      key: `assistant-${body.messageId}`,
      messageId: body.messageId,
      text: body.text,
      complete: body.complete
    })
    markWorking(draft)
    return
  }
  draft.items[index] = grownText(draft.items[index] as AssistantItem, body)
}

function foldThinking(
  draft: Draft,
  body: Extract<ChatBody, { type: 'assistantThinking' }>
): void {
  const index = lastIndexWhere(
    draft.items,
    (item) => item.kind === 'thinking' && item.messageId === body.messageId
  )
  if (index < 0) {
    draft.items.push({
      kind: 'thinking',
      key: `thinking-${body.messageId}`,
      messageId: body.messageId,
      text: body.text,
      complete: body.complete
    })
    markWorking(draft)
    return
  }
  draft.items[index] = grownText(draft.items[index] as ThinkingItem, body)
}

/** A partial after the complete text of the same message is a straggler and changes nothing. */
function grownText<Item extends AssistantItem | ThinkingItem>(
  item: Item,
  body: { text: string; complete: boolean }
): Item {
  if (item.complete) return item
  if (body.complete) return { ...item, text: body.text, complete: true }
  return { ...item, text: item.text + body.text }
}

function foldToolUse(draft: Draft, body: Extract<ChatBody, { type: 'toolUse' }>): void {
  const known = lastIndexWhere(
    draft.items,
    (item) => item.kind === 'tool' && item.toolUseId === body.toolUseId
  )
  if (known >= 0) return
  draft.items.push({
    kind: 'tool',
    key: `tool-${body.toolUseId}`,
    toolUseId: body.toolUseId,
    name: body.name,
    input: body.input,
    result: null,
    state: 'running'
  })
  markWorking(draft)
}

function foldToolResult(draft: Draft, body: Extract<ChatBody, { type: 'toolResult' }>): void {
  const index = lastIndexWhere(
    draft.items,
    (item) => item.kind === 'tool' && item.toolUseId === body.toolUseId
  )
  if (index < 0) return
  const tool = draft.items[index] as ToolItem
  draft.items[index] = {
    ...tool,
    result: { isError: body.isError, excerpt: body.excerpt, truncated: body.truncated },
    state: resultState(tool, body.isError)
  }
}

function resultState(tool: ToolItem, isError: boolean): ToolItem['state'] {
  if (tool.state === 'denied') return 'denied'
  return isError ? 'failed' : 'done'
}

function foldPermissionResolved(
  draft: Draft,
  body: Extract<ChatBody, { type: 'permissionResolved' }>
): void {
  const pending = draft.pending
  if (!pending || pending.requestId !== body.requestId) return
  draft.pending = null
  draft.turn = 'working'
  if (body.allowed) return
  const index = lastIndexWhere(
    draft.items,
    (item) => item.kind === 'tool' && item.toolUseId === pending.toolUseId
  )
  if (index < 0) return
  draft.items[index] = { ...(draft.items[index] as ToolItem), state: 'denied' }
}

function foldTurnFinished(
  draft: Draft,
  seq: number,
  body: Extract<ChatBody, { type: 'turnFinished' }>
): void {
  draft.items.push({
    kind: 'turn-footer',
    key: `turn-${seq}`,
    usage: body.usage,
    durationMs: body.durationMs,
    outcome: body.outcome
  })
  draft.tokens = addUsage(draft.tokens, body.usage)
  draft.turn = 'idle'
  draft.pending = null
}

function markWorking(draft: Draft): void {
  if (draft.turn === 'idle') draft.turn = 'working'
}

function addUsage(total: TurnUsage, turn: TurnUsage): TurnUsage {
  return {
    inputTokens: total.inputTokens + turn.inputTokens,
    cacheCreationInputTokens: total.cacheCreationInputTokens + turn.cacheCreationInputTokens,
    cacheReadInputTokens: total.cacheReadInputTokens + turn.cacheReadInputTokens,
    outputTokens: total.outputTokens + turn.outputTokens
  }
}
