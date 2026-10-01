import type { Task } from '@/model/catalog'

/** A project's tasks split three ways: live work, parked for later, finished. */
export interface PartitionedTasks {
  readonly active: Task[]
  readonly backlog: Task[]
  readonly filed: Task[]
}

export function partitionTasks(tasks: readonly Task[]): PartitionedTasks {
  const active: Task[] = []
  const backlog: Task[] = []
  const filed: Task[] = []
  for (const task of tasks) {
    if (task.status === 'DONE') filed.push(task)
    else if (task.status === 'BACKLOG') backlog.push(task)
    else active.push(task)
  }
  return { active, backlog, filed }
}
