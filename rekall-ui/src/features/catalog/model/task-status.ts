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
  TODO: 'bg-text-subtle',
  BLOCKED: 'bg-danger',
  BACKLOG: 'bg-border-strong',
  DONE: 'bg-safe'
}

export const TASK_STATUS_RING: Readonly<Record<TaskStatus, string>> = {
  IN_PROGRESS: 'bg-accent/20',
  TODO: 'bg-text-subtle/20',
  BLOCKED: 'bg-danger/20',
  BACKLOG: 'bg-border-strong/20',
  DONE: 'bg-safe/20'
}
