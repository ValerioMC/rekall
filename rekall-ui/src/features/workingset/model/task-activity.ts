export const TASK_ACTIVITIES = ['WORKING', 'EXECUTED', 'INACTIVE'] as const

export type TaskActivity = (typeof TASK_ACTIVITIES)[number]

export const TASK_ACTIVITY_LABEL: Readonly<Record<TaskActivity, string>> = {
  WORKING: 'A session is working',
  EXECUTED: 'Work was executed',
  INACTIVE: 'No activity'
}

export interface StepTally {
  readonly executed: number
  readonly total: number
}
