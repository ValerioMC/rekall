export type TurnOutcome = 'completed' | 'interrupted' | 'failed'

export type NoticeKind = 'apiRetry' | 'compaction' | 'warning'

export interface TurnUsage {
  readonly inputTokens: number
  readonly cacheCreationInputTokens: number
  readonly cacheReadInputTokens: number
  readonly outputTokens: number
}

/** What a tool call was asked to do; the tools the window shows in a way of their own are typed, the rest are `Other`. */
export type ToolInput =
  | { readonly tool: 'bash'; readonly command: string; readonly description: string | null }
  | {
      readonly tool: 'edit'
      readonly filePath: string
      readonly oldString: string
      readonly newString: string
      readonly replaceAll: boolean
    }
  | { readonly tool: 'write'; readonly filePath: string; readonly content: string }
  | { readonly tool: 'read'; readonly filePath: string }
  | {
      readonly tool: 'search'
      readonly name: string
      readonly pattern: string
      readonly path: string | null
    }
  | { readonly tool: 'other'; readonly name: string; readonly text: string }

/**
 * What a chat event says. A text or thinking event not yet `complete` carries only the words that
 * arrived since the last one; the `complete` one carries the whole text and replaces them.
 */
export type ChatBody =
  | { readonly type: 'userMessage'; readonly text: string }
  | {
      readonly type: 'assistantText'
      readonly messageId: string
      readonly text: string
      readonly complete: boolean
    }
  | {
      readonly type: 'assistantThinking'
      readonly messageId: string
      readonly text: string
      readonly complete: boolean
    }
  | {
      readonly type: 'toolUse'
      readonly toolUseId: string
      readonly name: string
      readonly input: ToolInput
    }
  | {
      readonly type: 'toolResult'
      readonly toolUseId: string
      readonly isError: boolean
      readonly excerpt: string
      readonly truncated: boolean
    }
  | {
      readonly type: 'permissionRequested'
      readonly requestId: string
      readonly toolUseId: string
      readonly name: string
      readonly input: ToolInput
    }
  | { readonly type: 'permissionResolved'; readonly requestId: string; readonly allowed: boolean }
  | {
      readonly type: 'turnFinished'
      readonly usage: TurnUsage
      readonly durationMs: number
      readonly outcome: TurnOutcome
    }
  | { readonly type: 'notice'; readonly kind: NoticeKind; readonly text: string }
  | { readonly type: 'ended'; readonly exitCode: number; readonly detail: string }

/** One thing that happened in a chat session; `seq` counts from 1 within the session. */
export type ChatEvent = ChatBody & { readonly seq: number }
