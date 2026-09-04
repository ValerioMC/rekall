import type { Task } from '@/model/catalog'

export interface PartitionedTasks {
  /** Everything still being worked: any status other than DONE. */
  readonly active: Task[]
  /** Finished work, filed out of the list until it is asked for. */
  readonly filed: Task[]
}

/**
 * Splits a task list into the work still open and the work that is done.
 *
 * The navigator shows finished tasks folded away by default: a project with thirty closed
 * tasks was burying the three that are live. The split keeps each side in the order it arrived
 * in, so opening the drawer does not reshuffle the rows underneath it.
 */
export function partitionTasks(tasks: readonly Task[]): PartitionedTasks {
  const active: Task[] = []
  const filed: Task[] = []
  for (const task of tasks) {
    if (task.status === 'DONE') filed.push(task)
    else active.push(task)
  }
  return { active, filed }
}
