import type { StatusPillTone } from '@/components/ui/StatusPill.vue'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskId } from '@/model/branded'

/** What a task row says about its task's state, beyond the task's status: the one word in its pill. */
export const TASK_ROW_KINDS = ['running', 'review', 'progress', 'todo', 'blocked', 'backlog', 'filed'] as const

export type TaskRowKind = (typeof TASK_ROW_KINDS)[number]

export const TASK_ROW_KIND_LABEL: Readonly<Record<TaskRowKind, string>> = {
  running: 'Session working',
  review: 'Awaiting review',
  progress: 'In progress',
  todo: 'To do',
  blocked: 'Blocked',
  backlog: 'Backlog',
  filed: 'Filed'
}

export const TASK_ROW_KIND_TONE: Readonly<Record<TaskRowKind, StatusPillTone>> = {
  running: 'running',
  review: 'review',
  progress: 'progress',
  todo: 'neutral',
  blocked: 'danger',
  backlog: 'neutral',
  filed: 'filed'
}

/** One task as a list shows it, with everything its columns and its tabs read already worked out. */
export interface TaskRow {
  readonly task: Task
  readonly kind: TaskRowKind
  /** Steps a session claimed and nobody has reviewed. */
  readonly claimedSteps: number
  /** Claimed steps, plus one when the task itself was handed in whole: what the review queue counts. */
  readonly needs: number
  readonly running: boolean
  readonly tokens: number
}

export type TokensOfTask = (taskId: TaskId) => number
