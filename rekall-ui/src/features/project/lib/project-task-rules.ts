import { compareText } from '@/features/catalog/lib/compare-text'
import { newestFirst, oldestFirst } from '@/features/catalog/model/task-code'
import type {
  ListedTask,
  ProjectTaskCounts,
  ProjectTaskSort,
  ProjectTaskSortKey,
  ProjectTaskView
} from '@/features/project/model/project-task-list'
import type { TaskRow, TaskRowKind } from '@/features/task/model/task-row'

const VIEW_RULES: Readonly<Record<ProjectTaskView, (row: TaskRow) => boolean>> = {
  needs: (row) => row.needs > 0,
  running: (row) => row.running,
  open: (row) => row.kind !== 'filed',
  filed: (row) => row.kind === 'filed',
  all: () => true
}

export function isInView(row: TaskRow, view: ProjectTaskView): boolean {
  return VIEW_RULES[view](row)
}

export function rowsOfView<Row extends TaskRow>(rows: readonly Row[], view: ProjectTaskView): Row[] {
  return rows.filter((row) => isInView(row, view))
}

/** Every word of the query has to appear in the title, the code or the step line, whatever the case. */
export function matchesQuery(row: ListedTask, query: string): boolean {
  const words = query.toLowerCase().split(/\s+/).filter((word) => word.length > 0)
  const haystack = `${row.task.title} ${row.task.code} ${row.line.text}`.toLowerCase()
  return words.every((word) => haystack.includes(word))
}

/**
 * What each tab counts. Needs me counts what the review queue counts (claimed steps plus tasks
 * handed in whole), so it equals the header's review segment when nothing narrows the rows.
 */
export function viewCounts(rows: readonly TaskRow[]): ProjectTaskCounts {
  return {
    needs: rows.reduce((sum, row) => sum + row.needs, 0),
    running: rowsOfView(rows, 'running').length,
    open: rowsOfView(rows, 'open').length,
    filed: rowsOfView(rows, 'filed').length,
    all: rows.length
  }
}

const STATUS_RANK: Readonly<Record<TaskRowKind, number>> = {
  filed: 0,
  backlog: 1,
  todo: 2,
  blocked: 3,
  progress: 4,
  review: 5,
  running: 6
}

function planShare(row: TaskRow): number {
  return row.task.stepCount > 0 ? (row.task.stepsDone + row.claimedSteps) / row.task.stepCount : -1
}

const ASCENDING: Readonly<Record<ProjectTaskSortKey, (a: ListedTask, b: ListedTask) => number>> = {
  task: (a, b) => oldestFirst(a.task, b.task),
  title: (a, b) => compareText(a.task.title, b.task.title),
  tag: (a, b) => compareText(a.task.tagName ?? '', b.task.tagName ?? ''),
  status: (a, b) => STATUS_RANK[a.kind] - STATUS_RANK[b.kind],
  plan: (a, b) => planShare(a) - planShare(b),
  notes: (a, b) => a.task.documentCount - b.task.documentCount,
  tokens: (a, b) => a.tokens - b.tokens,
  time: (a, b) => a.trackedSeconds - b.trackedSeconds,
  updated: (a, b) => a.task.updatedAt.localeCompare(b.task.updatedAt)
}

const TEXT_SORT_KEYS: readonly ProjectTaskSortKey[] = ['title', 'tag']

/** Pressing the key in use turns it round; a new key starts from A for text and from the most or latest otherwise. */
export function nextSort(current: ProjectTaskSort, key: ProjectTaskSortKey): ProjectTaskSort {
  if (current.key === key) return { key, descending: !current.descending }
  return { key, descending: !TEXT_SORT_KEYS.includes(key) }
}

/** Rows the key does not tell apart stay newest first, whichever way the key runs. */
export function sortRows(rows: readonly ListedTask[], sort: ProjectTaskSort): ListedTask[] {
  const direction = sort.descending ? -1 : 1
  return [...rows].sort((a, b) => direction * ASCENDING[sort.key](a, b) || newestFirst(a.task, b.task))
}
