import { stepIsComplete, stepIsDraft } from '@/features/steps/model/task-step'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TurnState } from '@/features/chat/model/timeline'
import type { StepTally, TaskActivity } from '@/features/workingset/model/task-activity'

export interface ActivityReading {
  readonly steps: readonly TaskStep[]
  readonly turn: TurnState | null
  readonly hasLiveSession: boolean
}

/**
 * Only a followed chat session has a folded timeline, and an in-app terminal has no
 * turn signal, so `turn` is null for those and `WORKING` then comes from a `RUNNING` step alone.
 */
export function taskActivity({ steps, turn, hasLiveSession }: ActivityReading): TaskActivity {
  if (steps.some((step) => step.state === 'RUNNING') || turn === 'working') return 'WORKING'
  if (hasLiveSession || steps.some((step) => stepIsComplete(step.state))) return 'EXECUTED'
  return 'INACTIVE'
}

export function stepTally(steps: readonly TaskStep[]): StepTally {
  const counted = steps.filter((step) => !stepIsDraft(step.state))
  return { executed: counted.filter((step) => stepIsComplete(step.state)).length, total: counted.length }
}
