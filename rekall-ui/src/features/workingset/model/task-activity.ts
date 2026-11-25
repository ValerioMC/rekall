export type TaskActivity = 'WORKING' | 'EXECUTED' | 'INACTIVE'

export interface StepTally {
  readonly executed: number
  readonly total: number
}
