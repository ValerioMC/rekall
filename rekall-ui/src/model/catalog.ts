import type {
  CompanyId,
  DocumentId,
  ProjectId,
  TaskId,
  TaskStepId,
  TimeEntryId,
  WrapupId
} from './branded'

export const DOCUMENT_KINDS = ['context', 'notes', 'architecture', 'report', 'other'] as const

export const PROJECT_STATUSES = ['ACTIVE', 'PAUSED', 'DONE'] as const
export const TASK_STATUSES = ['TODO', 'IN_PROGRESS', 'BLOCKED', 'DONE'] as const

export const TASK_STEP_STATES = ['OPEN', 'RUNNING', 'CLAIMED', 'DONE'] as const

export type ProjectStatus = (typeof PROJECT_STATUSES)[number]
export type TaskStatus = (typeof TASK_STATUSES)[number]
export type TaskStepState = (typeof TASK_STEP_STATES)[number]

export function stepIsComplete(state: TaskStepState): boolean {
  return state === 'CLAIMED' || state === 'DONE'
}

export const PROJECT_STATUS_LABEL: Readonly<Record<ProjectStatus, string>> = {
  ACTIVE: 'Active',
  PAUSED: 'Paused',
  DONE: 'Done'
}

export const TASK_STATUS_LABEL: Readonly<Record<TaskStatus, string>> = {
  IN_PROGRESS: 'In progress',
  TODO: 'To do',
  BLOCKED: 'Blocked',
  DONE: 'Done'
}

export const TASK_STATUS_ORDER: readonly TaskStatus[] = ['IN_PROGRESS', 'TODO', 'BLOCKED', 'DONE']

export const TASK_STATUS_COLOR: Readonly<Record<TaskStatus, string>> = {
  IN_PROGRESS: 'bg-accent',
  TODO: 'bg-text-subtle',
  BLOCKED: 'bg-danger',
  DONE: 'bg-safe'
}

export const TASK_STATUS_RING: Readonly<Record<TaskStatus, string>> = {
  IN_PROGRESS: 'bg-accent/20',
  TODO: 'bg-text-subtle/20',
  BLOCKED: 'bg-danger/20',
  DONE: 'bg-safe/20'
}

export const PROJECT_STATUS_COLOR: Readonly<Record<ProjectStatus, string>> = {
  ACTIVE: 'bg-accent',
  PAUSED: 'bg-text-subtle',
  DONE: 'bg-safe'
}

export interface Company {
  readonly id: CompanyId
  readonly name: string
  readonly description: string | null
  readonly projectCount: number
  readonly taskCount: number
  readonly updatedAt: string
}

export interface Project {
  readonly id: ProjectId
  readonly label: string
  readonly title: string
  readonly status: ProjectStatus
  readonly description: string | null
  readonly blueprintMarkdown: string | null
  readonly repoFolder: string | null
  readonly companyId: CompanyId
  readonly companyName: string
  readonly taskCount: number
  readonly anchor: string
  readonly updatedAt: string
}

export interface Task {
  readonly id: TaskId
  readonly label: string
  readonly title: string
  readonly status: TaskStatus
  readonly description: string | null
  readonly autoWrapup: boolean
  readonly wrapupDirective: string | null
  readonly projectId: ProjectId
  readonly projectLabel: string
  readonly projectTitle: string
  readonly companyName: string
  readonly projectRepoFolder: string | null
  readonly documentCount: number
  readonly stepCount: number
  readonly stepsDone: number
  readonly hasWrapup: boolean
  readonly reviewState: TaskStepState
  readonly reviewActive: boolean
  readonly claimedAt: string | null
  readonly acceptedAt: string | null
  readonly reviewNote: string | null
  readonly anchor: string
  readonly updatedAt: string
}

/**
 * The task-scoped review line, as it arrives on the step SSE feed under the
 * `task-review` event. Meaningful only while `reviewActive` is true, i.e. the
 * task has no checklist.
 */
export interface TaskReview {
  readonly taskId: TaskId
  readonly reviewState: TaskStepState
  readonly reviewActive: boolean
  readonly claimedAt: string | null
  readonly acceptedAt: string | null
  readonly reviewNote: string | null
}

export interface TaskStep {
  readonly id: TaskStepId
  readonly taskId: TaskId
  readonly title: string
  readonly bodyMarkdown: string | null
  readonly state: TaskStepState
  readonly done: boolean
  readonly runningAt: string | null
  readonly claimedAt: string | null
  readonly doneAt: string | null
  readonly position: number
  readonly createdAt: string
  readonly updatedAt: string
}

export function stepCompletedAt(step: TaskStep): string | null {
  return step.claimedAt ?? step.doneAt
}

export interface RekallDocument {
  readonly id: DocumentId
  readonly title: string
  readonly kind: string
  readonly bodyMarkdown: string
  readonly tasks: readonly TaskRef[]
  readonly updatedAt: string
}

export type WrapupAuthor = 'CLAUDE' | 'HAND'

export const WRAPUP_AUTHOR_LABEL: Readonly<Record<WrapupAuthor, string>> = {
  CLAUDE: 'Claude',
  HAND: 'you'
}

export interface Wrapup {
  readonly id: WrapupId
  readonly taskId: TaskId
  readonly taskLabel: string
  readonly taskTitle: string
  readonly projectLabel: string
  readonly anchor: string
  readonly bodyMarkdown: string
  readonly writtenBy: WrapupAuthor
  readonly createdAt: string
  readonly updatedAt: string
}

/**
 * A wrapup write or delete, as it arrives on the step SSE feed under the
 * `wrapup` event. `wrapup` is null when `deleted` is true.
 */
export interface WrapupStreamEvent {
  readonly taskId: TaskId
  readonly wrapup: Wrapup | null
  readonly deleted: boolean
}

export interface TimeEntry {
  readonly id: TimeEntryId
  readonly taskId: TaskId
  readonly taskLabel: string
  readonly taskTitle: string
  readonly projectLabel: string
  readonly anchor: string
  readonly startedAt: string
  readonly stoppedAt: string | null
  readonly createdAt: string
  readonly updatedAt: string
}

export interface TaskRef {
  readonly id: TaskId
  readonly label: string
  readonly title: string
  readonly projectLabel: string
  readonly projectTitle: string
  readonly companyName: string
  readonly anchor: string
}

export function slugify(raw: string): string {
  return raw
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, '-')
    .replace(/[._-]{2,}/g, '-')
    .replace(/^[._-]+|[._-]+$/g, '')
}

export function anchorOf(record: Company | Project | Task): string {
  return 'anchor' in record ? record.anchor : `company:${record.name}`
}
