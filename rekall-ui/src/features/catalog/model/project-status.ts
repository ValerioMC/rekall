import type { StatusPillTone } from '@/components/ui/StatusPill.vue'

export const PROJECT_STATUSES =['ACTIVE', 'PAUSED', 'DONE', 'ARCHIVED'] as const

export type ProjectStatus = (typeof PROJECT_STATUSES)[number]

export const PROJECT_STATUS_LABEL: Readonly<Record<ProjectStatus, string>> = {
  ACTIVE: 'Active',
  PAUSED: 'Paused',
  DONE: 'Done',
  ARCHIVED: 'Archived'
}

export type ProjectStatusChoice = ProjectStatus | 'ALL'

export const PROJECT_STATUS_TONE: Readonly<Record<ProjectStatus, StatusPillTone>> = {
  ACTIVE: 'progress',
  PAUSED: 'neutral',
  DONE: 'done',
  ARCHIVED: 'filed'
}

export const PROJECT_STATUS_COLOR: Readonly<Record<ProjectStatus, string>> = {
  ACTIVE: 'bg-accent',
  PAUSED: 'bg-text-subtle',
  DONE: 'bg-safe',
  ARCHIVED: 'bg-border-strong'
}
