import { isEpic } from '@/features/catalog/model/catalog'
import type { Task } from '@/features/catalog/model/catalog'
import { newestFirst } from '@/features/catalog/model/task-code'
import type { TaskStatus } from '@/features/catalog/model/task-status'

/** An epic and the tasks that belong to it, whatever their status: they are listed under it and nowhere else. */
export interface EpicBranch {
  readonly epic: Task
  readonly children: Task[]
}

/**
 * A project's tasks split four ways, each newest first: the epics with the tasks they gather, then
 * the tasks that belong to no epic (or to one the filter hides): live work, parked for later, finished.
 */
export interface PartitionedTasks {
  readonly epics: EpicBranch[]
  readonly active: Task[]
  readonly backlog: Task[]
  readonly filed: Task[]
}

export function partitionTasks(tasks: readonly Task[]): PartitionedTasks {
  const sorted = [...tasks].sort(newestFirst)
  const epics = sorted.filter(isEpic).map((epic): EpicBranch => ({ epic, children: [] }))
  const branchById = new Map(epics.map((branch) => [branch.epic.id, branch]))
  const active: Task[] = []
  const backlog: Task[] = []
  const filed: Task[] = []
  for (const task of sorted) {
    if (isEpic(task)) continue
    const branch = task.epicId === null ? undefined : branchById.get(task.epicId)
    if (branch) branch.children.push(task)
    else if (task.status === 'DONE') filed.push(task)
    else if (task.status === 'BACKLOG') backlog.push(task)
    else active.push(task)
  }
  return { epics, active, backlog, filed }
}

/** The branches whose epic has this status: the epics a status section lists before its tasks. */
export function branchesWithStatus(epics: readonly EpicBranch[], status: TaskStatus): EpicBranch[] {
  return epics.filter((branch) => branch.epic.status === status)
}
