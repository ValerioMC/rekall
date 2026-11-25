import type { TaskId, TaskStepId } from '@/model/branded'

/** What a chat session held of its context window at the end of its last turn on a task, or on one of its steps. */
export interface ContextFill {
  readonly stepId: TaskStepId | null
  readonly usedTokens: number
  readonly windowTokens: number
  readonly percent: number
  readonly recordedAt: string
}

/** The task's whole list of fills after a turn changed one. */
export interface ContextFillStreamEvent {
  readonly taskId: TaskId
  readonly fills: readonly ContextFill[]
}

/** What a badge speaks for: a task's own fill, or one of its steps'. */
export interface ContextFillSubject {
  readonly taskId: TaskId
  readonly stepId: TaskStepId | null
}

export type ContextFillLevel = 'neutral' | 'warning' | 'danger'
