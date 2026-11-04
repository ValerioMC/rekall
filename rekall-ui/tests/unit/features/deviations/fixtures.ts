import type { TaskDeviation } from '@/features/deviations/model/deviation'
import type { TaskId } from '@/model/branded'

export const taskId = 't1' as TaskId

export function deviation(stepNumber: number, extra: Partial<TaskDeviation> = {}): TaskDeviation {
  return {
    id: `00000000-0000-0000-0000-00000000000${stepNumber}`,
    taskId,
    stepNumber,
    stepTitle: `Title of step ${stepNumber}`,
    bodyMarkdown: `What step ${stepNumber} did differently.`,
    createdAt: '2026-10-01T10:00:00Z',
    updatedAt: '2026-10-01T10:00:00Z',
    ...extra
  }
}
