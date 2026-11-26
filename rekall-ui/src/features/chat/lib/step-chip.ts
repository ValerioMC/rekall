import { groupBySession } from '@/features/steps/lib/step-sessions'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

export interface SessionStep {
  readonly name: string
  readonly coverage: string | null
}

/** The step a session is on, named "Step 3 · Write the CSV file"; none when it is on no step of `steps`. */
export function sessionStep(stepId: TaskStepId | null, steps: readonly TaskStep[]): SessionStep | null {
  const index = steps.findIndex((step) => step.id === stepId)
  if (!stepId || index < 0) return null
  return { name: `Step ${index + 1} · ${steps[index]!.title}`, coverage: coverage(stepId, steps) }
}

function coverage(stepId: TaskStepId, steps: readonly TaskStep[]): string | null {
  const group = groupBySession(steps).find((session) => session.steps.some((step) => step.id === stepId))
  if (!group || group.steps.length < 2) return null
  const first = steps.indexOf(group.steps[0]!) + 1
  const last = steps.indexOf(group.steps[group.steps.length - 1]!) + 1
  return `session covers steps ${first} to ${last}`
}
