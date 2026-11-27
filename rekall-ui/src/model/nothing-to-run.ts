import { stepIsComplete, stepIsDraft } from '@/features/steps/model/task-step'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'

export const NOTHING_TO_RUN_MESSAGE =
  'Nothing to run: every step is claimed or done, and the rest are drafts. Promote a draft to run it.'

/** A checklist with no open or running step is spent; a task without one is spent once its review is claimed or done and only drafts remain. */
export function nothingToRun(
  task: Pick<Task, 'id' | 'reviewActive' | 'reviewState'> | null,
  steps: readonly Pick<TaskStep, 'taskId' | 'state'>[]
): boolean {
  if (!task) return false
  const own = steps.filter((step) => step.taskId === task.id)
  const checklist = own.filter((step) => !stepIsDraft(step.state))
  if (checklist.length > 0) return checklist.every((step) => stepIsComplete(step.state))
  const hasDraft = own.length > 0
  return hasDraft && task.reviewActive && stepIsComplete(task.reviewState)
}
