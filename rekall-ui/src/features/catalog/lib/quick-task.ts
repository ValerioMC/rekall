import type { Task } from '@/features/catalog/model/catalog'

export const QUICK_TASK_HELP =
  'Describe the change. Claude does it in one session; it is accepted automatically when the wrapup is written.'

export function withoutQuickTasks(tasks: readonly Task[], hidden: boolean): readonly Task[] {
  return hidden ? tasks.filter((task) => !task.quick) : tasks
}

const FOLDER_TAIL_SEGMENTS = 2

/** The end of a folder path, where the project's own name is: all the room a composer chip has. */
export function folderTail(folder: string): string {
  const segments = folder.split('/').filter((segment) => segment !== '')
  return segments.length <= FOLDER_TAIL_SEGMENTS ? folder : `…/${segments.slice(-FOLDER_TAIL_SEGMENTS).join('/')}`
}
