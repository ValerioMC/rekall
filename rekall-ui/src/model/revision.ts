import type { TaskId } from '@/model/branded'
import type { WrapupAuthor } from '@/model/catalog'

/** Which text of a task a revision keeps an earlier version of. */
export type RevisionKind = 'WRAPUP' | 'DESCRIPTION'

/** An earlier version of a task's wrapup or description, kept when something replaced or deleted it. */
export interface TaskRevision {
  readonly id: string
  readonly taskId: TaskId
  readonly kind: RevisionKind
  readonly bodyMarkdown: string
  /** Known for a wrapup, absent for a description. */
  readonly writtenBy: WrapupAuthor | null
  /** When the version kept here was written, when that is known. */
  readonly writtenAt: string | null
  /** When it stopped being the current text. */
  readonly replacedAt: string
}

/** What a restore wrote back as the current text. */
export interface RestoredRevision {
  readonly taskId: TaskId
  readonly kind: RevisionKind
  readonly bodyMarkdown: string
}
