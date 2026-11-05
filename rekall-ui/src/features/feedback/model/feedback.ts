import type { FeedbackId, TaskId } from '@/model/branded'

export type FeedbackKind = 'REVISION' | 'IMPLEMENTATION' | 'PLAN'
export type FeedbackState = 'PENDING' | 'APPLIED'

/** The kinds a person writes in the composer; a revision comes from saving the description. */
export type WrittenFeedbackKind = Exclude<FeedbackKind, 'REVISION'>

/**
 * One message of a task's thread. A revision holds the description before (`quotedMarkdown`) and
 * after (`bodyMarkdown`) a rewrite; the other kinds hold only the body. It stays pending until a
 * session replies, and `seq` numbers a task's entries from 1.
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
}

export function isPending(entry: Feedback): boolean {
  return entry.state === 'PENDING'
}
