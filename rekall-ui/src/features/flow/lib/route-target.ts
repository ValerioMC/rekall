import type { TaskStepId } from '@/model/branded'

/** What a route leads to besides a step: the end of the flow. */
export const FLOW_END = 'flow:end' as const

/** Where a route leads: a step of the task, or the end of the flow. */
export type RouteTarget = TaskStepId | typeof FLOW_END

export function isFlowEnd(target: RouteTarget): target is typeof FLOW_END {
  return target === FLOW_END
}
