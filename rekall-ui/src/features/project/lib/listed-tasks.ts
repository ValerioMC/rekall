import { stepLine } from '@/features/project/lib/task-step-line'
import { trackedSeconds } from '@/features/task/lib/tracked-seconds'
import type { ListedTask } from '@/features/project/model/project-task-list'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskRow } from '@/features/task/model/task-row'
import type { TimeEntry } from '@/features/time/model/time-entry'
import type { TaskId } from '@/model/branded'

function byTask<Item extends { readonly taskId: TaskId }>(items: readonly Item[]): Map<TaskId, Item[]> {
  const grouped = new Map<TaskId, Item[]>()
  for (const item of items) {
    const group = grouped.get(item.taskId)
    if (group) group.push(item)
    else grouped.set(item.taskId, [item])
  }
  return grouped
}

/** Each row with its step line and the time tracked on it up to `nowMs`. `steps` and `entries` may hold every task's. */
export function listedTasks(
  rows: readonly TaskRow[],
  steps: readonly TaskStep[],
  entries: readonly TimeEntry[],
  nowMs: number
): ListedTask[] {
  const stepsOf = byTask(steps)
  const entriesOf = byTask(entries)
  return rows.map((row) => ({
    ...row,
    line: stepLine(row, stepsOf.get(row.task.id) ?? []),
    trackedSeconds: trackedSeconds(entriesOf.get(row.task.id) ?? [], nowMs)
  }))
}
