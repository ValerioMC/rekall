import { stepIsComplete, stepIsDraft } from '@/model/catalog'
import type { Task, TaskStep } from '@/model/catalog'
import type { TaskStepId } from '@/model/branded'

export const NOTHING_TO_RUN_MESSAGE =
  'Nothing to run: every step is claimed or done, and the rest are drafts. Promote a draft to run it.'

/**
 * Whether a session launched on the whole task would find nothing to execute. A checklist with no
 * open or running step is spent; a task with no checklist is spent once its review is claimed or
 * done and only drafts remain. A launch on one chosen step is always deliberate, so never refused.
 */
export function nothingToRun(
  task: Pick<Task, 'id' | 'reviewActive' | 'reviewState'> | null,
  steps: readonly Pick<TaskStep, 'taskId' | 'state'>[],
  stepId: TaskStepId | null
): boolean {
  if (!task || stepId) return false
  const own = steps.filter((step) => step.taskId === task.id)
  const checklist = own.filter((step) => !stepIsDraft(step.state))
  if (checklist.length > 0) return checklist.every((step) => stepIsComplete(step.state))
  const hasDraft = own.length > 0
  return hasDraft && task.reviewActive && stepIsComplete(task.reviewState)
}
