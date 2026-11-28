import type { TaskRow } from '@/features/task/model/task-row'

export const PROJECT_TASK_VIEWS = ['needs', 'running', 'open', 'filed', 'all'] as const

export type ProjectTaskView = (typeof PROJECT_TASK_VIEWS)[number]

export const PROJECT_TASK_VIEW_LABEL: Readonly<Record<ProjectTaskView, string>> = {
  needs: 'Needs me',
  running: 'Running',
  open: 'Open',
  filed: 'Filed',
  all: 'All'
}

export type ProjectTaskCounts = Readonly<Record<ProjectTaskView, number>>

export const PROJECT_TASK_SORT_KEYS = ['task', 'title', 'tag', 'status', 'plan', 'notes', 'tokens', 'time', 'updated'] as const

export type ProjectTaskSortKey = (typeof PROJECT_TASK_SORT_KEYS)[number]

export interface ProjectTaskSort {
  readonly key: ProjectTaskSortKey
  readonly descending: boolean
}

export const DEFAULT_PROJECT_TASK_SORT: ProjectTaskSort = { key: 'task', descending: true }

export const PROJECT_TASK_DENSITIES = ['comfortable', 'compact'] as const

export type ProjectTaskDensity = (typeof PROJECT_TASK_DENSITIES)[number]

export const TASK_PAGE_SIZES = [25, 50, 100] as const

export type TaskPageSize = (typeof TASK_PAGE_SIZES)[number]

export const TASK_FILTER_KEYS = ['status', 'tag', 'epic', 'type'] as const

export type TaskFilterKey = (typeof TASK_FILTER_KEYS)[number]

export const TASK_FILTER_LABEL: Readonly<Record<TaskFilterKey, string>> = {
  status: 'Status',
  tag: 'Tag',
  epic: 'Epic',
  type: 'Type'
}

/** The values picked per filter; an empty list lets every task through. */
export type TaskFilters = Readonly<Record<TaskFilterKey, readonly string[]>>

export const NO_TASK_FILTERS: TaskFilters = { status: [], tag: [], epic: [], type: [] }

export interface TaskFilterOption {
  readonly value: string
  readonly label: string
  readonly count: number
}

/** The columns a person can take away, and that give way as the list narrows. */
export const OPTIONAL_TASK_COLUMNS = ['step', 'tag', 'delivery', 'notes', 'tokens', 'time'] as const

export type OptionalTaskColumn = (typeof OPTIONAL_TASK_COLUMNS)[number]

export type TaskStepLineTone = 'running' | 'claimed' | 'next' | 'summary'

/** What a task is on, in one line: the step that needs attention, else the start of its description. */
export interface TaskStepLine {
  readonly tone: TaskStepLineTone
  readonly lead: string | null
  readonly text: string
}

/** A task row with what only a project's list shows. */
export interface ListedTask extends TaskRow {
  readonly line: TaskStepLine
  readonly trackedSeconds: number
}

/** What a project's list shows in one place: a task alone, or an epic with the tasks it gathers under it. */
export type ProjectTaskEntry<Row extends TaskRow = ListedTask> =
  | { readonly kind: 'task'; readonly row: Row; readonly epicTitle: string | null }
  | { readonly kind: 'epic'; readonly row: Row; readonly members: readonly Row[] }
