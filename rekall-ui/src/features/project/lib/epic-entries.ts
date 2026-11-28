import { isEpic } from '@/features/catalog/model/catalog'
import type { ProjectTaskEntry } from '@/features/project/model/project-task-list'
import type { TaskRow } from '@/features/task/model/task-row'
import type { TaskId } from '@/model/branded'

function membersOf<Row extends TaskRow>(epic: Row, rows: readonly Row[]): Row[] {
  return rows.filter((row) => row.task.epicId === epic.task.id)
}

/**
 * Gathers each epic's tasks under it, at the epic's place in the list. A task whose epic is not
 * among the rows (another view, another tab) stays where it is and names its epic instead.
 */
export function epicEntries<Row extends TaskRow>(
  rows: readonly Row[],
  titleOfEpic: (id: TaskId) => string | null
): ProjectTaskEntry<Row>[] {
  const epicIds = new Set(rows.filter((row) => isEpic(row.task)).map((row) => row.task.id))
  return rows.flatMap((row): ProjectTaskEntry<Row>[] => {
    if (isEpic(row.task)) return [{ kind: 'epic', row, members: membersOf(row, rows) }]
    const epicId = row.task.epicId
    if (epicId !== null && epicIds.has(epicId)) return []
    return [{ kind: 'task', row, epicTitle: epicId === null ? null : titleOfEpic(epicId) }]
  })
}

/** How many tasks an entry lists: an epic counts itself and the tasks under it. */
export function tasksInEntry(entry: ProjectTaskEntry<TaskRow>): number {
  return entry.kind === 'epic' ? 1 + entry.members.length : 1
}
