import type { TaskId, TaskStepId } from '@/model/branded'

/** The commit logged for a task, or one of its steps: the tip of the project's own repo folder. */
export interface CommitReference {
  readonly id: string
  readonly taskId: TaskId
  readonly stepId: TaskStepId | null
  readonly stepTitle: string | null
  readonly commitHash: string
  readonly comment: string
  /** Chosen in the console to travel with `/rk`: hash, subject and diff land in the session's context. */
  readonly inContext: boolean
  readonly createdAt: string
}

/** One line of the project's recent git log: what the picker lists so a commit can be logged by hand. */
export interface RecentCommit {
  readonly hash: string
  readonly subject: string
  readonly committedAt: string
}
