import type { TaskId, WrapupId } from '../../../model/branded'

export type WrapupAuthor = 'CLAUDE' | 'HAND'

export const WRAPUP_AUTHOR_LABEL: Readonly<Record<WrapupAuthor, string>> = {
  CLAUDE: 'Claude',
  HAND: 'you'
}

export interface Wrapup {
  readonly id: WrapupId
  readonly taskId: TaskId
  readonly taskCode: string
  readonly taskTitle: string
  readonly projectLabel: string
  readonly anchor: string
  readonly bodyMarkdown: string
  readonly writtenBy: WrapupAuthor
  readonly createdAt: string
  readonly updatedAt: string
}

/** A wrapup write or delete (`wrapup` SSE event); `wrapup` is null when `deleted` is true. */
export interface WrapupStreamEvent {
  readonly taskId: TaskId
  readonly wrapup: Wrapup | null
  readonly deleted: boolean
}
