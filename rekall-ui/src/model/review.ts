import type { TaskId, TaskStepId } from '@/model/branded'
import type { Task, TaskStep } from '@/model/catalog'

/**
 * One piece of work a session claimed and nobody has reviewed: a claimed step, or a task with no
 * checklist whose wrapup claimed it.
 */
export interface ReviewItem {
  /** Stable across refreshes: the step's id, or the task's for a stepless task. */
  readonly key: string
  readonly taskId: TaskId
  readonly stepId: TaskStepId | null
  readonly taskTitle: string
  readonly stepTitle: string | null
  readonly anchor: string
  readonly claimedAt: string | null
}

/** Everything waiting for review across every task, the longest-waiting first. */
export function reviewQueue(tasks: readonly Task[], steps: readonly TaskStep[]): ReviewItem[] {
  const byId = new Map(tasks.map((task) => [task.id, task]))
  const items: ReviewItem[] = []
  for (const step of steps) {
    if (step.state !== 'CLAIMED') continue
    const task = byId.get(step.taskId)
    if (!task) continue
    items.push({
      key: step.id,
      taskId: task.id,
      stepId: step.id,
      taskTitle: task.title,
      stepTitle: step.title,
      anchor: task.anchor,
      claimedAt: step.claimedAt
    })
  }
  for (const task of tasks) {
    if (!task.reviewActive || task.reviewState !== 'CLAIMED') continue
    items.push({
      key: task.id,
      taskId: task.id,
      stepId: null,
      taskTitle: task.title,
      stepTitle: null,
      anchor: task.anchor,
      claimedAt: task.claimedAt
    })
  }
  return items.sort((a, b) => (a.claimedAt ?? '').localeCompare(b.claimedAt ?? ''))
}
