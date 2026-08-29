import type { CompanyId, DocumentId, ProjectId, TaskId, TimeEntryId, WrapupId } from './branded'

/** Free-form on the server; these are the ones the editor offers. */
export const DOCUMENT_KINDS = ['context', 'notes', 'architecture', 'report', 'other'] as const

export const PROJECT_STATUSES = ['ACTIVE', 'PAUSED', 'DONE'] as const
export const TASK_STATUSES = ['TODO', 'IN_PROGRESS', 'BLOCKED', 'DONE'] as const

export type ProjectStatus = (typeof PROJECT_STATUSES)[number]
export type TaskStatus = (typeof TASK_STATUSES)[number]

/** Enum constants are shouted; a person reading a list is not. */
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

/** The order the navigator groups tasks in, so the number keys match what is on screen. */
export const TASK_STATUS_ORDER: readonly TaskStatus[] = ['IN_PROGRESS', 'TODO', 'BLOCKED', 'DONE']

export const TASK_STATUS_COLOR: Readonly<Record<TaskStatus, string>> = {
  IN_PROGRESS: 'bg-accent',
  TODO: 'bg-text-subtle',
  BLOCKED: 'bg-danger',
  DONE: 'bg-safe'
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

/**
 * A project as the console holds it.
 *
 * `label` and `title` are both here because they answer different questions. The title is what
 * the row reads as; the label is what has to be typed after `/rk` to load it, and it is the one
 * that has to stay put.
 */
export interface Project {
  readonly id: ProjectId
  readonly label: string
  readonly title: string
  readonly status: ProjectStatus
  readonly description: string | null
  readonly blueprintMarkdown: string | null
  readonly companyId: CompanyId
  readonly companyName: string
  readonly taskCount: number
  /** Server-built, so the client never assembles an anchor the server would not resolve. */
  readonly anchor: string
  readonly updatedAt: string
}

export interface Task {
  readonly id: TaskId
  readonly label: string
  readonly title: string
  readonly status: TaskStatus
  readonly description: string | null
  readonly projectId: ProjectId
  readonly projectLabel: string
  readonly projectTitle: string
  readonly companyName: string
  readonly documentCount: number
  /** Whether this task has said what it currently is. The body lives on the wrapup itself. */
  readonly hasWrapup: boolean
  readonly anchor: string
  readonly updatedAt: string
}

export interface RekallDocument {
  readonly id: DocumentId
  readonly title: string
  readonly kind: string
  readonly bodyMarkdown: string
  /** Every task this note is attached to. A note is on at least one, often on several. */
  readonly tasks: readonly TaskRef[]
  readonly updatedAt: string
}

/** Who last wrote a wrapup. Not decoration: it says whose words the next write will replace. */
export type WrapupAuthor = 'CLAUDE' | 'HAND'

export const WRAPUP_AUTHOR_LABEL: Readonly<Record<WrapupAuthor, string>> = {
  CLAUDE: 'Claude',
  HAND: 'you'
}

/**
 * What a task's implementation looks like now.
 *
 * One per task, replaced whole rather than appended to, and deliberately not a note: a note is
 * something you learned and can be attached to several tasks, a wrapup is the state of one
 * task and there is only ever one answer to that.
 */
export interface Wrapup {
  readonly id: WrapupId
  readonly taskId: TaskId
  readonly taskLabel: string
  readonly taskTitle: string
  readonly projectLabel: string
  /** What you would type after `/rk` to load the task this describes. */
  readonly anchor: string
  readonly bodyMarkdown: string
  readonly writtenBy: WrapupAuthor
  readonly createdAt: string
  readonly updatedAt: string
}

/**
 * One sitting of work on a task.
 *
 * `stoppedAt` is null for exactly as long as the session is open, and only one session per task
 * may be in that state at a time — different tasks can each have one open at once, tracked in
 * parallel.
 */
export interface TimeEntry {
  readonly id: TimeEntryId
  readonly taskId: TaskId
  readonly taskLabel: string
  readonly taskTitle: string
  readonly projectLabel: string
  /** What you would type after `/rk` to load the task this session was spent on. */
  readonly anchor: string
  readonly startedAt: string
  readonly stoppedAt: string | null
  readonly createdAt: string
  readonly updatedAt: string
}

/** Enough of a task to name it and link to it, as returned alongside a note. */
export interface TaskRef {
  readonly id: TaskId
  readonly label: string
  readonly title: string
  readonly projectLabel: string
  readonly projectTitle: string
  readonly companyName: string
  /** What you would type after `/rk` to load it. */
  readonly anchor: string
}

/**
 * The same narrowing the server applies, so the field can show the anchor before it is saved.
 *
 * Duplicated on purpose rather than round-tripped: a label preview that arrives one request
 * later is a label preview nobody reads. The server remains the authority, and the response
 * carries the stored value back.
 */
export function slugify(raw: string): string {
  return raw
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, '-')
    .replace(/[._-]{2,}/g, '-')
    .replace(/^[._-]+|[._-]+$/g, '')
}

/** What you would type after `/rk` to load this record. */
export function anchorOf(record: Company | Project | Task): string {
  return 'anchor' in record ? record.anchor : `company:${record.name}`
}
