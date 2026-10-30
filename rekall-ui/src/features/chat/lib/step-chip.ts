import { groupBySession } from '@/features/steps/lib/step-sessions'
import type { TaskStep } from '@/features/catalog/model/catalog'
import type { TaskStepId } from '@/model/branded'

/** "Step 3 of 7 · session covers steps 3 to 5"; "Task" when the session is on no step, none when the step is not in `steps`. */
export function stepChip(stepId: TaskStepId | null, steps: readonly TaskStep[]): string | null {
  if (!stepId) return 'Task'
  const index = steps.findIndex((step) => step.id === stepId)
  if (index < 0) return null
  const own = `Step ${index + 1} of ${steps.length}`
  const group = groupBySession(steps).find((session) => session.steps.some((step) => step.id === stepId))
  if (!group || group.steps.length < 2) return own
  const first = steps.indexOf(group.steps[0]!) + 1
  const last = steps.indexOf(group.steps[group.steps.length - 1]!) + 1
  return `${own} · session covers steps ${first} to ${last}`
}
