import type { TaskStep } from '@/features/steps/model/task-step'

export interface SessionChange {
  readonly step: TaskStep
  readonly startsSession: boolean
}

export interface StepMove {
  /** Where the step lands among the checklist once it is lifted out. */
  readonly position: number
  readonly sessionChanges: readonly SessionChange[]
}

/**
 * Taking a step that opens a session out of its place hands the opening to the step behind it, so
 * the steps that followed do not slide into the session above.
 */
export function openerHandover(steps: readonly TaskStep[], leaving: TaskStep): SessionChange[] {
  const index = steps.findIndex((step) => step.id === leaving.id)
  const successor = steps[index + 1]
  if (index <= 0 || !leaving.startsSession || !successor || successor.startsSession) return []
  return [{ step: successor, startsSession: true }]
}

/**
 * Puts `moving` right before or after `target`. It joins the session it lands in, and lands at the
 * head of that session when it goes in front of the step that opens it. Answers null when the step
 * would not change place.
 */
export function planStepMove(steps: readonly TaskStep[], moving: TaskStep, target: TaskStep, after: boolean): StepMove | null {
  if (moving.id === target.id) return null
  const flags = new Map<string, boolean>(steps.map((step) => [step.id, step.startsSession]))
  openerHandover(steps, moving).forEach((change) => flags.set(change.step.id, change.startsSession))

  const remaining = steps.filter((step) => step.id !== moving.id)
  const targetIndex = remaining.findIndex((step) => step.id === target.id)
  const position = after ? targetIndex + 1 : targetIndex
  if (position === steps.findIndex((step) => step.id === moving.id)) return null

  const targetOpens = flags.get(target.id) === true
  flags.set(moving.id, !after && targetOpens && position > 0)
  if (!after && targetOpens) flags.set(target.id, false)

  const sessionChanges = steps
    .filter((step) => flags.get(step.id) !== step.startsSession)
    .map((step) => ({ step, startsSession: flags.get(step.id) === true }))
  return { position, sessionChanges }
}
