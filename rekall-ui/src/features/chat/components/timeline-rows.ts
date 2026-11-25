import type { Component } from 'vue'
import ChatCommandOutput from './ChatCommandOutput.vue'
import ChatEnded from './ChatEnded.vue'
import ChatMessage from './ChatMessage.vue'
import ChatNotice from './ChatNotice.vue'
import ChatSessionStart from './ChatSessionStart.vue'
import ChatThinking from './ChatThinking.vue'
import ChatTurnFooter from './ChatTurnFooter.vue'
import ToolCallCard from './ToolCallCard.vue'
import type { TimelineItem } from '@/features/chat/model/timeline'

export interface RowContext {
  readonly followingNext: boolean
  readonly innerCalls: number
}

interface Row<Item extends TimelineItem> {
  readonly component: Component
  readonly props: (item: Item, context: RowContext) => Record<string, unknown>
}

type Rows = { readonly [Kind in TimelineItem['kind']]: Row<Extract<TimelineItem, { kind: Kind }>> }

const ROWS: Rows = {
  session: {
    component: ChatSessionStart,
    props: (item) => ({ model: item.model, cwd: item.cwd, permissionMode: item.permissionMode })
  },
  'command-output': {
    component: ChatCommandOutput,
    props: (item) => ({ command: item.command, text: item.text })
  },
  user: { component: ChatMessage, props: (item) => ({ role: 'user', text: item.text, complete: true }) },
  assistant: {
    component: ChatMessage,
    props: (item) => ({ role: 'assistant', text: item.text, complete: item.complete })
  },
  thinking: { component: ChatThinking, props: (item) => ({ text: item.text, complete: item.complete }) },
  tool: { component: ToolCallCard, props: (item, context) => ({ call: item, innerCalls: context.innerCalls }) },
  notice: { component: ChatNotice, props: (item) => ({ kind: item.noticeKind, text: item.text }) },
  'turn-footer': {
    component: ChatTurnFooter,
    props: (item) => ({ usage: item.usage, durationMs: item.durationMs, outcome: item.outcome })
  },
  ended: {
    component: ChatEnded,
    props: (item, context) => ({ exitCode: item.exitCode, detail: item.detail, followingNext: context.followingNext })
  }
}

export interface ResolvedRow {
  readonly component: Component
  readonly props: Record<string, unknown>
}

export function resolveRow(item: TimelineItem, context: RowContext): ResolvedRow {
  const row = ROWS[item.kind] as Row<TimelineItem>
  return { component: row.component, props: row.props(item, context) }
}
