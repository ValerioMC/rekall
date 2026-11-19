export const PROJECT_STATUSES = ['ACTIVE', 'PAUSED', 'DONE', 'ARCHIVED'] as const

export type ProjectStatus = (typeof PROJECT_STATUSES)[number]

export const PROJECT_STATUS_LABEL: Readonly<Record<ProjectStatus, string>> = {
  ACTIVE: 'Active',
  PAUSED: 'Paused',
  DONE: 'Done',
  ARCHIVED: 'Archived'
}

export const PROJECT_STATUS_COLOR: Readonly<Record<ProjectStatus, string>> = {
  ACTIVE: 'bg-accent',
  PAUSED: 'bg-text-subtle',
  DONE: 'bg-safe',
  ARCHIVED: 'bg-border-strong'
}
