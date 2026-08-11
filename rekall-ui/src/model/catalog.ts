import type { DocumentId, EnvironmentId, ProjectId, TaskId } from './branded'

/** Free-form on the server; these are the ones the editor offers. */
export const DOCUMENT_KINDS = ['context', 'notes', 'architecture', 'report', 'other'] as const

export const PROJECT_STATUSES = ['ACTIVE', 'PAUSED', 'DONE'] as const
export const TASK_STATUSES = ['TODO', 'IN_PROGRESS', 'BLOCKED', 'DONE'] as const

export type ProjectStatus = (typeof PROJECT_STATUSES)[number]
export type TaskStatus = (typeof TASK_STATUSES)[number]

export interface Project {
  readonly id: ProjectId
  readonly name: string
  readonly status: ProjectStatus
  readonly description: string | null
  readonly taskCount: number
  readonly updatedAt: string
}

export interface Task {
  readonly id: TaskId
  readonly name: string
  readonly status: TaskStatus
  readonly description: string | null
  readonly projectId: ProjectId
  readonly projectName: string
  readonly environmentId: EnvironmentId | null
  readonly environmentLabel: string | null
  readonly updatedAt: string
}

export interface Environment {
  readonly id: EnvironmentId
  readonly label: string
  readonly namespace: string | null
  readonly kubeconfigPath: string | null
  readonly updatedAt: string
}

export interface RekallDocument {
  readonly id: DocumentId
  readonly title: string
  readonly kind: string
  readonly bodyMarkdown: string
  /** The anchor of whatever owns it, e.g. `task:code-validator`. */
  readonly owner: string
  readonly position: number
  readonly updatedAt: string
}

/** What you would type after `/rk` to load this record. */
export function anchorOf(record: Project | Task | Environment): string {
  if ('taskCount' in record) return `project:${record.name}`
  if ('projectId' in record) return `task:${record.name}`
  return `environment:${record.label}`
}
