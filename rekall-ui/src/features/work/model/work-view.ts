import type { StatusPillTone } from '@/components/ui/StatusPill.vue'
import type { Task } from '@/features/catalog/model/catalog'
import type { ProjectStatus } from '@/features/catalog/model/project-status'
import type { ProjectId, TaskId } from '@/model/branded'

export const WORK_VIEWS = ['needs', 'running', 'open', 'filed'] as const

export type WorkView = (typeof WORK_VIEWS)[number]

export const WORK_VIEW_LABEL: Readonly<Record<WorkView, string>> = {
  needs: 'Needs me',
  running: 'Running',
  open: 'Open',
  filed: 'Filed'
}

export const WORK_SORTS = ['task', 'needs', 'tokens', 'updated'] as const

export type WorkSort = (typeof WORK_SORTS)[number]

export const WORK_DENSITIES = ['comfortable', 'compact'] as const

export type WorkDensity = (typeof WORK_DENSITIES)[number]

/** What a ledger row says about its task's state, beyond the task's status: the one word in its pill. */
export const WORK_KINDS = ['running', 'review', 'progress', 'todo', 'blocked', 'backlog', 'filed'] as const

export type WorkKind = (typeof WORK_KINDS)[number]

export const WORK_KIND_LABEL: Readonly<Record<WorkKind, string>> = {
  running: 'Session working',
  review: 'Awaiting review',
  progress: 'In progress',
  todo: 'To do',
  blocked: 'Blocked',
  backlog: 'Backlog',
  filed: 'Filed'
}

export const WORK_KIND_TONE: Readonly<Record<WorkKind, StatusPillTone>> = {
  running: 'running',
  review: 'review',
  progress: 'progress',
  todo: 'neutral',
  blocked: 'danger',
  backlog: 'neutral',
  filed: 'filed'
}

/** One task as the ledger lists it, with everything its columns and its tabs read already worked out. */
export interface WorkRow {
  readonly task: Task
  readonly kind: WorkKind
  /** Steps a session claimed and nobody has reviewed. */
  readonly claimedSteps: number
  /** Claimed steps, plus one when the task itself was handed in whole: what the review queue counts. */
  readonly needs: number
  readonly running: boolean
  readonly tokens: number
}

/** The tasks of one project that a view lists, with the finished ones folded into a line of their own. */
export interface WorkGroup {
  readonly projectId: ProjectId
  readonly projectTitle: string
  readonly companyName: string
  readonly status: ProjectStatus
  readonly icon: string
  readonly iconImageVersion: string | null
  readonly rows: readonly WorkRow[]
  readonly filed: readonly WorkRow[]
  readonly needs: number
  readonly tokens: number
}

export type WorkCounts = Readonly<Record<WorkView, number>>

export type TokensOfTask = (taskId: TaskId) => number

export type WorkLaneId = 'todo' | 'progress' | 'review' | 'filed'

/** One column of the board: the rows of the tasks that stand in that state. */
export interface WorkLane {
  readonly id: WorkLaneId
  readonly label: string
  readonly rows: readonly WorkRow[]
}

/** What a project's ledger lists in one place: a task alone, or an epic with the tasks it gathers under it. */
export type WorkEntry =
  | { readonly kind: 'task'; readonly row: WorkRow; readonly epicTitle: string | null }
  | { readonly kind: 'epic'; readonly row: WorkRow; readonly members: readonly WorkRow[] }
