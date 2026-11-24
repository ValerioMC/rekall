import type { ProjectStatus } from '@/features/catalog/model/project-status'
import type { TaskRow } from '@/features/task/model/task-row'
import type { ProjectId } from '@/model/branded'

export const PROJECT_TASK_VIEWS = ['needs', 'running', 'open', 'filed'] as const

export type ProjectTaskView = (typeof PROJECT_TASK_VIEWS)[number]

export const PROJECT_TASK_VIEW_LABEL: Readonly<Record<ProjectTaskView, string>> = {
  needs: 'Needs me',
  running: 'Running',
  open: 'Open',
  filed: 'Filed'
}

export const PROJECT_TASK_SORTS = ['task', 'needs', 'tokens', 'updated'] as const

export type ProjectTaskSort = (typeof PROJECT_TASK_SORTS)[number]

export const PROJECT_TASK_DENSITIES = ['comfortable', 'compact'] as const

export type ProjectTaskDensity = (typeof PROJECT_TASK_DENSITIES)[number]

/** The tasks of one project that a view lists, with the finished ones folded into a line of their own. */
export interface TasksOfProject {
  readonly projectId: ProjectId
  readonly projectTitle: string
  readonly companyName: string
  readonly status: ProjectStatus
  readonly icon: string
  readonly iconImageVersion: string | null
  readonly rows: readonly TaskRow[]
  readonly filed: readonly TaskRow[]
  readonly needs: number
  readonly tokens: number
}

export type ProjectTaskCounts = Readonly<Record<ProjectTaskView, number>>

/** What a project's list shows in one place: a task alone, or an epic with the tasks it gathers under it. */
export type ProjectTaskEntry =
  | { readonly kind: 'task'; readonly row: TaskRow; readonly epicTitle: string | null }
  | { readonly kind: 'epic'; readonly row: TaskRow; readonly members: readonly TaskRow[] }
