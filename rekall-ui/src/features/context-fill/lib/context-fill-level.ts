import type { ContextFill, ContextFillLevel } from '../model/context-fill'
import type { TaskStepId } from '@/model/branded'

const WARNING_FROM_PERCENT = 60
const DANGER_FROM_PERCENT = 85

export function contextFillLevel(percent: number): ContextFillLevel {
  if (percent >= DANGER_FROM_PERCENT) return 'danger'
  if (percent >= WARNING_FROM_PERCENT) return 'warning'
  return 'neutral'
}

/** The fill of a step, or of the task itself when `stepId` is null; none when no turn has recorded one. */
export function fillOf(fills: readonly ContextFill[], stepId: TaskStepId | null): ContextFill | null {
  return fills.find((fill) => fill.stepId === stepId) ?? null
}
