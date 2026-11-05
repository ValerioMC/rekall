import type { FeedbackId, TaskId } from '@/model/branded'

export type FeedbackKind = 'REVISION' | 'IMPLEMENTATION' | 'PLAN'
export type FeedbackState = 'PENDING' | 'RUNNING' | 'APPLIED'

/** The kinds a person writes in the composer; a revision comes from saving the description. */
export type WrittenFeedbackKind = Exclude<FeedbackKind, 'REVISION'>

/**
 * One message of a task's thread. A revision holds the description before (`quotedMarkdown`) and
 * after (`bodyMarkdown`) a rewrite; the other kinds hold only the body. It waits (`PENDING`) until
 * Run revision takes it into a numbered `pass` (`RUNNING`), and is `APPLIED` once a session replies.
 * `followsSeq` names the applied entry it adds detail to, and `seq` numbers a task's entries from 1.
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
  readonly followsSeq: number | null
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
