export const TASK_STATUSES = ['BACKLOG', 'TODO', 'IN_PROGRESS', 'BLOCKED', 'DONE'] as const

export type TaskStatus = (typeof TASK_STATUSES)[number]

export const TASK_STATUS_LABEL: Readonly<Record<TaskStatus, string>> = {
  IN_PROGRESS: 'In progress',
  TODO: 'To do',
  BLOCKED: 'Blocked',
  BACKLOG: 'Backlog',
  DONE: 'Done'
}

export const TASK_STATUS_ORDER: readonly TaskStatus[] = ['IN_PROGRESS', 'TODO', 'BLOCKED', 'BACKLOG', 'DONE']

export const TASK_STATUS_COLOR: Readonly<Record<TaskStatus, string>> = {
  IN_PROGRESS: 'bg-accent',
  TODO: 'bg-anchor',
  BLOCKED: 'bg-danger',
  BACKLOG: 'bg-text-subtle',
  DONE: 'bg-safe'
}
