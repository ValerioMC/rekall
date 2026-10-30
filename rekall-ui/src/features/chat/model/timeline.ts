import type { NoticeKind, ToolInput, TurnOutcome, TurnUsage } from './chat-event'

export type TurnState = 'idle' | 'working' | 'waiting'

export interface UserItem {
  readonly kind: 'user'
  readonly key: string
  readonly text: string
}

export interface AssistantItem {
  readonly kind: 'assistant'
  readonly key: string
  readonly messageId: string
  readonly text: string
  readonly complete: boolean
}

export interface ThinkingItem {
  readonly kind: 'thinking'
  readonly key: string
  readonly messageId: string
  readonly text: string
  readonly complete: boolean
}

export interface ToolResult {
  readonly isError: boolean
  readonly excerpt: string
  readonly truncated: boolean
}

/** `running` until its result arrives, then `done` or `failed`; `denied` when the person refused it. */
export type ToolState = 'running' | 'done' | 'failed' | 'denied'

export interface ToolItem {
  readonly kind: 'tool'
  readonly key: string
  readonly toolUseId: string
  readonly name: string
  readonly input: ToolInput
  readonly result: ToolResult | null
  readonly state: ToolState
}

export interface NoticeItem {
  readonly kind: 'notice'
  readonly key: string
  readonly noticeKind: NoticeKind
  readonly text: string
}

export interface TurnFooterItem {
  readonly kind: 'turn-footer'
  readonly key: string
  readonly usage: TurnUsage
  readonly durationMs: number
  readonly outcome: TurnOutcome
}

export interface EndedItem {
  readonly kind: 'ended'
  readonly key: string
  readonly exitCode: number
  readonly detail: string
}

export type TimelineItem =
  | UserItem
  | AssistantItem
  | ThinkingItem
  | ToolItem
  | NoticeItem
  | TurnFooterItem
  | EndedItem

export interface PendingPermission {
  readonly requestId: string
  readonly toolUseId: string
  readonly name: string
  readonly input: ToolInput
}

/** What the window shows of one chat session, folded from its events. */
export interface Timeline {
  readonly items: readonly TimelineItem[]
  readonly lastSeq: number
  readonly turn: TurnState
  readonly pending: PendingPermission | null
  readonly tokens: TurnUsage
  readonly ended: boolean
}
