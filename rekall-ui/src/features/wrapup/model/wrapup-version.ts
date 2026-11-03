import type { WrapupAuthor } from '@/features/catalog/model/catalog'
import type { TaskId, WrapupVersionId } from '@/model/branded'

/** A past text of a task's wrapup, without the text. */
export interface WrapupVersion {
  readonly id: WrapupVersionId
  readonly taskId: TaskId
  readonly number: number
  readonly writtenBy: WrapupAuthor
  readonly writtenAt: string
  readonly length: number
  readonly preview: string
  readonly commitHash: string | null
}

export interface WrapupVersionDetail extends WrapupVersion {
  readonly bodyMarkdown: string
}
