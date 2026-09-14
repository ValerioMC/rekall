import type { TaskId, TaskStepId } from '@/model/branded'

/** The commit logged for a task, or one of its steps: the tip of the project's own repo folder. */
export interface CommitReference {
  readonly id: string
  readonly taskId: TaskId
  readonly stepId: TaskStepId | null
  readonly stepTitle: string | null
  readonly commitHash: string
  readonly comment: string
  readonly createdAt: string
}
