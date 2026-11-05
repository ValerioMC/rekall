/** A task's branch as the review shows it: what it holds beyond the project's current branch. */
export interface TaskBranch {
  readonly branch: string
  readonly commitsAhead: number
  readonly changedFiles: readonly string[]
  /** Null when git cannot say whether merging would conflict. */
  readonly mergesCleanly: boolean | null
  /** The worktree's folder while it exists. */
  readonly worktreeFolder: string | null
  readonly hasUnsavedChanges: boolean
}
