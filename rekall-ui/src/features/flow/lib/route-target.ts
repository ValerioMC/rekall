import type { TaskStepId } from '@/model/branded'

/** What a route leads to besides a step: the end of the flow. */
export const FLOW_END = 'flow:end' as const

/** Where a route leads: a step of the task (or of a deployment graph), or the end of the flow. */
export type RouteTarget<StepId extends string = TaskStepId> = StepId | typeof FLOW_END

export function isFlowEnd<StepId extends string>(target: RouteTarget<StepId>): target is typeof FLOW_END {
  return target === FLOW_END
}
