import type { Task } from '@/features/catalog/model/catalog'
import type { ProjectId } from '@/model/branded'

export const QUICK_TASK_HELP =
  'Describe the change. Claude does it in one session; it is accepted automatically when the wrapup is written.'

export const QUICK_TASK_BUSY_REASON =
  'A quick task is already being worked on this project. It can be started again once its wrapup is written.'

/** Mirrors the server's limit: a quick task with a session on it. One whose session ended is back to open and does not count. */
export function activeQuickTask(tasks: readonly Task[], projectId: ProjectId): Task | null {
  return (
    tasks.find(
      (task) =>
        task.projectId === projectId &&
        task.quick &&
        task.reviewState === 'RUNNING'
    ) ?? null
  )
}

export function withoutQuickTasks(tasks: readonly Task[], hidden: boolean): readonly Task[] {
  return hidden ? tasks.filter((task) => !task.quick) : tasks
}
