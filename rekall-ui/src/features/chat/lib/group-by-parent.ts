import type { TimelineItem, ToolItem } from '../model/timeline'

export interface GroupedItems {
  readonly top: readonly TimelineItem[]
  readonly inner: ReadonlyMap<string, readonly TimelineItem[]>
}

/**
 * Splits the items into the conversation and, for each Agent call, what its subagent did, in the
 * order it happened. An item whose parent is not an Agent call in the list stays in the conversation.
 */
export function groupByParent(items: readonly TimelineItem[]): GroupedItems {
  const agents = agentCallIds(items)
  const top: TimelineItem[] = []
  const inner = new Map<string, TimelineItem[]>()
  for (const item of items) {
    const parent = parentOf(item)
    if (parent === null || !agents.has(parent)) {
      top.push(item)
      continue
    }
    inner.set(parent, [...(inner.get(parent) ?? []), item])
  }
  return { top, inner }
}

export function countCalls(items: readonly TimelineItem[]): number {
  return items.filter((item) => item.kind === 'tool').length
}

function agentCallIds(items: readonly TimelineItem[]): ReadonlySet<string> {
  const calls = items.filter((item): item is ToolItem => item.kind === 'tool' && item.input.tool === 'agent')
  return new Set(calls.map((call) => call.toolUseId))
}

function parentOf(item: TimelineItem): string | null {
  switch (item.kind) {
    case 'assistant':
    case 'thinking':
    case 'tool':
      return item.parentToolUseId
    default:
      return null
  }
}
