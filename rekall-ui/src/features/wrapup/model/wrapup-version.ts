import type { WrapupAuthor } from '@/features/wrapup/model/wrapup'
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

/** Whether the files a wrapup names changed after it was written. Anything git cannot say is `unknown`. */
export type WrapupFreshness =
  | { readonly state: 'fresh' }
  | { readonly state: 'unknown' }
  | { readonly state: 'possiblyStale'; readonly changedPaths: readonly string[]; readonly commits: readonly string[] }
