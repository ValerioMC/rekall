import type { FeedbackId, TaskId } from '@/model/branded'

/** What a revision is applied to: the work already done, or the plan of steps. */
export type FeedbackKind = 'IMPLEMENTATION' | 'PLAN'
export type FeedbackState = 'PENDING' | 'RUNNING' | 'APPLIED'

/**
 * One revision of a task's description: the text before (`quotedMarkdown`) and after
 * (`bodyMarkdown`) a saved change, applied to the work or the plan (`kind`). It waits (`PENDING`)
 * until Run revision takes it into a numbered `pass` (`RUNNING`), and is `APPLIED` once a session
 * replies. `seq` numbers a task's revisions from 1.
 */
export interface Feedback {
  readonly id: FeedbackId
  readonly taskId: TaskId
  readonly seq: number
  readonly kind: FeedbackKind
  readonly state: FeedbackState
  readonly quotedMarkdown: string | null
  readonly bodyMarkdown: string
  readonly replyMarkdown: string | null
  readonly createdAt: string
  readonly appliedAt: string | null
  readonly pass: number | null
  readonly takenAt: string | null
}

export function isWaiting(entry: Feedback): boolean {
  return entry.state === 'PENDING'
}

export function isRunning(entry: Feedback): boolean {
  return entry.state === 'RUNNING'
}

export function isOpen(entry: Feedback): boolean {
  return entry.state !== 'APPLIED'
}
