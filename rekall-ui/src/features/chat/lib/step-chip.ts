import { groupBySession } from '@/features/steps/lib/step-sessions'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskStepId } from '@/model/branded'

export interface SessionStep {
  readonly name: string
  readonly coverage: string | null
}

/** "Step 3 of 7 · session covers steps 3 to 5"; "Task" when the session is on no step, none when the step is not in `steps`. */
export function stepChip(stepId: TaskStepId | null, steps: readonly TaskStep[]): string | null {
  if (!stepId) return 'Task'
  const index = steps.findIndex((step) => step.id === stepId)
  if (index < 0) return null
  const own = `Step ${index + 1} of ${steps.length}`
  const covered = coverage(stepId, steps)
  return covered ? `${own} · ${covered}` : own
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
