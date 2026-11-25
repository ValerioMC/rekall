import type { ContextFill } from '@/features/context-fill/model/context-fill'
import type { TaskId, TaskStepId } from '@/model/branded'

export const TASK = 'aaaaaaaa-0000-4000-8000-000000000001' as TaskId
export const STEP = 'bbbbbbbb-0000-4000-8000-000000000001' as TaskStepId
export const OTHER_STEP = 'bbbbbbbb-0000-4000-8000-000000000002' as TaskStepId

export function fill(stepId: TaskStepId | null, percent: number): ContextFill {
  return {
    stepId,
    usedTokens: percent * 2000,
    windowTokens: 200000,
    percent,
    recordedAt: new Date().toISOString()
  }
}
