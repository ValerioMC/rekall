import type { Task } from '@/features/catalog/model/catalog'

export const QUICK_TASK_HELP =
  'Describe the change. Claude does it in one session; it is accepted automatically when the wrapup is written.'

export function withoutQuickTasks(tasks: readonly Task[], hidden: boolean): readonly Task[] {
  return hidden ? tasks.filter((task) => !task.quick) : tasks
}
