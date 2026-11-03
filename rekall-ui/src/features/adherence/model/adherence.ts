import type { TaskId } from '@/model/branded'

/** How often a flow entered one step, against the budget the step allows. */
export interface StepVisits {
  readonly stepNumber: number
  readonly title: string
  readonly visits: number
  readonly maxVisits: number | null
  readonly budgetSpent: boolean
}

/** The numbers every adherence view shares. `firstTimeShare` is a fraction, null until a step is done. */
export interface AdherenceFigures {
  readonly stepsTotal: number
  readonly stepsDone: number
  readonly stepsClaimed: number
  readonly acceptedFirstTime: number
  readonly sentBack: number
  readonly passesTotal: number
  readonly firstTimeShare: number | null
  readonly deviationNotes: number
}

export interface PlanAdherence extends AdherenceFigures {
  readonly deviationSteps: readonly number[]
  readonly visits: readonly StepVisits[]
}

export interface TaskSentBack {
  readonly taskId: TaskId
  readonly label: string
  readonly title: string
  readonly sentBack: number
  readonly passesTotal: number
}

export interface ProjectAdherence extends AdherenceFigures {
  readonly mostSentBack: readonly TaskSentBack[]
}
