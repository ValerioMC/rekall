import { planStepAlone } from '@/features/steps/lib/step-order'
import type { StepSession } from '@/features/steps/lib/step-sessions'
import type { TaskStep } from '@/features/steps/model/task-step'

export interface SessionPlace {
  readonly number: number
  /** The step already runs in it. */
  readonly current: boolean
  /** None of its steps has been started, so it can take one more. */
  readonly takes: boolean
}

/** Where the menu of an open step can send it. */
export interface StepPlaces {
  readonly canMoveUp: boolean
  readonly canMoveDown: boolean
  readonly sessions: readonly SessionPlace[]
  /** The number of the session the step would run in front of, on its own; null when that changes nothing. */
  readonly aloneBeforeSession: number | null
  readonly aloneAtEnd: boolean
}

function neighbourIsOpen(steps: readonly TaskStep[], step: TaskStep, by: number): boolean {
  return step.state === 'OPEN' && steps[steps.indexOf(step) + by]?.state === 'OPEN'
}

/** The sessions as a step sees them; a draft, which runs in none, passes null. */
export function sessionPlaces(sessions: readonly StepSession[], step: TaskStep | null): SessionPlace[] {
  return sessions.map((session) => ({
    number: session.number,
    current: step !== null && session.steps.some((candidate) => candidate.id === step.id),
    takes: session.steps.every((candidate) => candidate.state === 'OPEN')
  }))
}

export function placesOf(steps: readonly TaskStep[], sessions: readonly StepSession[], step: TaskStep): StepPlaces {
  const own = sessions.find((session) => session.steps.some((candidate) => candidate.id === step.id))
  return {
    canMoveUp: neighbourIsOpen(steps, step, -1),
    canMoveDown: neighbourIsOpen(steps, step, 1),
    sessions: sessionPlaces(sessions, step),
    aloneBeforeSession: own && planStepAlone(steps, step, own.steps[0] ?? null) ? own.number : null,
    aloneAtEnd: planStepAlone(steps, step, null) !== null
  }
}
