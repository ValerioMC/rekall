import type { Task } from '@/model/catalog'

export interface PartitionedTasks {
  readonly active: Task[]
  readonly filed: Task[]
}

export function partitionTasks(tasks: readonly Task[]): PartitionedTasks {
  const active: Task[] = []
  const filed: Task[] = []
  for (const task of tasks) {
    if (task.status === 'DONE') filed.push(task)
    else active.push(task)
  }
  return { active, filed }
}
