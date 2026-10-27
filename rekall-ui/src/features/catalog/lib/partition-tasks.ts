import { isEpic } from '@/features/catalog/model/catalog'
import type { Task } from '@/features/catalog/model/catalog'

/** A project's tasks split four ways: the epics, live work, parked for later, finished. */
export interface PartitionedTasks {
  readonly epics: Task[]
  readonly active: Task[]
  readonly backlog: Task[]
  readonly filed: Task[]
}

export function partitionTasks(tasks: readonly Task[]): PartitionedTasks {
  const epics: Task[] = []
  const active: Task[] = []
  const backlog: Task[] = []
  const filed: Task[] = []
  for (const task of tasks) {
    if (isEpic(task)) epics.push(task)
    else if (task.status === 'DONE') filed.push(task)
    else if (task.status === 'BACKLOG') backlog.push(task)
    else active.push(task)
  }
  return { epics, active, backlog, filed }
}
