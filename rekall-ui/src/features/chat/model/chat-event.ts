export type TurnOutcome = 'completed' | 'interrupted' | 'failed'

export type NoticeKind = 'apiRetry' | 'compaction' | 'warning'

export interface TurnUsage {
  readonly inputTokens: number
  readonly cacheCreationInputTokens: number
  readonly cacheReadInputTokens: number
  readonly outputTokens: number
}

export type TodoStatus = 'pending' | 'inProgress' | 'completed'

export interface TodoItem {
  readonly content: string
  readonly status: TodoStatus
}

/** What a turn's last API call held of the model's context window. */
export interface TurnContext {
  readonly usedTokens: number
  readonly windowTokens: number
}

export type FileChangeKind = 'add' | 'delete' | 'update'

/** One file of a change set. `diff` is a unified diff for an update, and the file's whole text otherwise. */
export interface FileChange {
  readonly path: string
  readonly kind: FileChangeKind
  readonly movedTo: string | null
  readonly diff: string
}

/** What a tool call was asked to do; the tools the chat shows in a way of their own are typed, the rest are `Other`. */
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
  | { readonly tool: 'fileChanges'; readonly changes: readonly FileChange[] }
  | { readonly tool: 'read'; readonly filePath: string }
  | {
      readonly tool: 'search'
      readonly name: string
      readonly pattern: string
      readonly path: string | null
    }
  | { readonly tool: 'todos'; readonly items: readonly TodoItem[] }
  | {
      readonly tool: 'agent'
      readonly description: string
      readonly subagentType: string | null
      readonly prompt: string
    }
  | { readonly tool: 'web'; readonly name: string; readonly target: string }
  | {
      readonly tool: 'mcp'
      readonly server: string
      readonly toolName: string
      readonly text: string
    }
  | { readonly tool: 'other'; readonly name: string; readonly text: string }

/**
 * What a chat event says. A text or thinking event not yet `complete` carries only the words that
 * arrived since the last one; the `complete` one carries the whole text and replaces them.
 * `parentToolUseId` is the Agent call a subagent's event belongs to, `null` in the main conversation.
 */
export type ChatBody =
  | {
      readonly type: 'sessionStarted'
      readonly model: string
      readonly cwd: string
      readonly permissionMode: string
      readonly version: string
    }
  | { readonly type: 'commandOutput'; readonly command: string; readonly text: string }
  | { readonly type: 'userMessage'; readonly text: string }
  | {
      readonly type: 'assistantText'
      readonly messageId: string
      readonly text: string
      readonly complete: boolean
      readonly parentToolUseId: string | null
    }
  | {
      readonly type: 'assistantThinking'
      readonly messageId: string
      readonly text: string
      readonly complete: boolean
      readonly parentToolUseId: string | null
    }
  | {
      readonly type: 'toolUse'
      readonly toolUseId: string
      readonly name: string
      readonly input: ToolInput
      readonly parentToolUseId: string | null
    }
  | {
      readonly type: 'toolResult'
      readonly toolUseId: string
      readonly isError: boolean
      readonly excerpt: string
      readonly truncated: boolean
      readonly summary: string | null
      readonly parentToolUseId: string | null
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
      readonly costUsd: number | null
      readonly context: TurnContext | null
    }
  | { readonly type: 'notice'; readonly kind: NoticeKind; readonly text: string }
  | { readonly type: 'ended'; readonly exitCode: number; readonly detail: string }

/** One thing that happened in a chat session; `seq` counts from 1 within the session. */
export type ChatEvent = ChatBody & { readonly seq: number }
