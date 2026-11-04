import type { TaskId } from '@/model/branded'
import type { TaskStep } from '@/features/catalog/model/catalog'

/** Where a task's work departed from its plan, for one step. A session writes it; it is not a note. */
export interface TaskDeviation {
  readonly id: string
  readonly taskId: TaskId
  /** The step's place on the checklist, counted from 1, drafts not counted. */
  readonly stepNumber: number
  /** The title of the step at that place today; absent when the checklist no longer reaches it. */
  readonly stepTitle: string | null
  readonly bodyMarkdown: string
  readonly createdAt: string
  readonly updatedAt: string
}

/** The number a step goes by on the checklist, which is what a deviation names; a draft has none. */
export function stepNumberOf(step: TaskStep, steps: readonly TaskStep[]): number | null {
  if (step.state === 'DRAFT') return null
  const checklist = steps.filter((candidate) => candidate.state !== 'DRAFT')
  const place = checklist.findIndex((candidate) => candidate.id === step.id)
  return place === -1 ? null : place + 1
}

/** The deviation after `current` in step order (`by` = 1) or before it (`by` = -1); null at either end. */
export function neighbourOf(
  deviations: readonly TaskDeviation[],
  current: number,
  by: 1 | -1
): TaskDeviation | null {
  const place = deviations.findIndex((deviation) => deviation.stepNumber === current)
  return deviations[place + by] ?? null
}
