import type { TaskStepId } from '@/model/branded'

export type ChangeKind = 'ADDED' | 'MODIFIED' | 'DELETED' | 'RENAMED'

export type DiffLineKind = 'CONTEXT' | 'ADDED' | 'REMOVED'

/** A removed line has no new number and an added one has no old number. */
export interface DiffLine {
  readonly kind: DiffLineKind
  readonly oldNumber: number | null
  readonly newNumber: number | null
  readonly text: string
}

export interface DiffHunk {
  readonly oldStart: number
  readonly oldLines: number
  readonly newStart: number
  readonly newLines: number
  readonly context: string
  readonly lines: readonly DiffLine[]
}

export interface ChangedFile {
  readonly path: string
  readonly oldPath: string | null
  readonly kind: ChangeKind
  readonly binary: boolean
  readonly additions: number
  readonly deletions: number
  readonly hunks: readonly DiffHunk[]
}

/** What one commit logged against the task changed; `available` is false when no patch could be read for it. */
export interface CommitChanges {
  readonly referenceId: string
  readonly commitHash: string
  readonly subject: string
  readonly stepId: TaskStepId | null
  readonly loggedAt: string
  readonly available: boolean
  readonly truncated: boolean
  readonly files: readonly ChangedFile[]
}

/** What the project's folder holds that no commit does yet. */
export interface WorkingTreeChanges {
  readonly truncated: boolean
  readonly files: readonly ChangedFile[]
}

/** What the task changed in one repository: the project's own folder (`componentId` null) or a component's. */
export interface RepositoryChanges {
  readonly componentId: string | null
  readonly name: string
  readonly commits: readonly CommitChanges[]
  readonly workingTree: WorkingTreeChanges | null
}

/** The repositories the task touched, the project's own folder first. */
export interface TaskChanges {
  readonly repositories: readonly RepositoryChanges[]
}
