import type { CompanyId, DocumentId, ProjectId, TaskId } from './branded'

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
