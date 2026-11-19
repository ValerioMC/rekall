import type { ConnectionId, StepLinkId, StepVisitId, TaskId, TaskStepId } from '../../../model/branded'

export const TASK_STEP_STATES = ['DRAFT', 'OPEN', 'RUNNING', 'CLAIMED', 'DONE'] as const

export type TaskStepState = (typeof TASK_STEP_STATES)[number]

export function stepIsComplete(state: TaskStepState): boolean {
  return state === 'CLAIMED' || state === 'DONE'
}

export function stepIsDraft(state: TaskStepState): boolean {
  return state === 'DRAFT'
}

/** The task-scoped review line (`task-review` SSE event); meaningful only while `reviewActive` is true. */
export interface TaskReview {
  readonly taskId: TaskId
  readonly reviewState: TaskStepState
  readonly reviewActive: boolean
  readonly claimedAt: string | null
  readonly acceptedAt: string | null
  readonly autoAccepted: boolean
}

/** An earlier pass at a step: what it worked from, kept when its claim was sent back. */
export interface StepPass {
  readonly detailMarkdown: string | null
  readonly sentBackAt: string
}

/** A route out of a step: when it ends with `outcome`, the work moves to `toStepId`, or ends when it is null. */
export interface StepLink {
  readonly id: StepLinkId
  readonly fromStepId: TaskStepId
  readonly toStepId: TaskStepId | null
  readonly outcome: string
  readonly position: number
}

/** One entry of the flow into a step; `ordinal` is its move number across the task. */
export interface StepVisit {
  readonly id: StepVisitId
  readonly stepId: TaskStepId
  readonly ordinal: number
  readonly enteredAt: string
  readonly leftAt: string | null
  readonly outcome: string | null
  /** What the session saw when it left the step, read by the step the flow went to next. */
  readonly report: string | null
}

export interface TaskStep {
  readonly id: TaskStepId
  readonly taskId: TaskId
  readonly title: string
  readonly bodyMarkdown: string | null
  readonly state: TaskStepState
  readonly done: boolean
  readonly runningAt: string | null
  readonly claimedAt: string | null
  readonly doneAt: string | null
  readonly position: number
  /** Where the step was dragged to on the flow canvas; null leaves it to the automatic layout. */
  readonly flowX: number | null
  readonly flowY: number | null
  /** How many times a flow may enter the step; null sets no limit. */
  readonly maxVisits: number | null
  /** Whether the step opens a fresh Claude session; false joins the one the step before it runs in. */
  readonly startsSession: boolean
  /** The company connection the step reaches, carried over from a deployment graph. */
  readonly connectionId: ConnectionId | null
  readonly passes: readonly StepPass[]
  /** Routes leaving this step; a task whose steps carry any is a flow. */
  readonly links: readonly StepLink[]
  /** Every entry of the flow into this step, oldest first; an open one is where it stands. */
  readonly visits: readonly StepVisit[]
  readonly createdAt: string
  readonly updatedAt: string
}

export function stepCompletedAt(step: TaskStep): string | null {
  return step.claimedAt ?? step.doneAt
}
