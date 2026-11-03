import type { CheckResult, DoneCheck } from '@/features/check/model/check'
import type { CheckResultId, DoneCheckId, ProjectId, TaskId, TaskStepId } from '@/model/branded'

export const PROJECT = 'p-1' as ProjectId
export const TASK = 't-1' as TaskId
export const STEP = 's-1' as TaskStepId

export function check(id: string, position: number, stepId: TaskStepId | null = null, label = id): DoneCheck {
  return {
    id: id as DoneCheckId,
    projectId: PROJECT,
    stepId,
    label,
    command: 'cargo test --workspace',
    folder: null,
    timeoutSeconds: 600,
    position,
    createdAt: '2026-11-03T10:00:00Z',
    updatedAt: '2026-11-03T10:00:00Z'
  }
}

export function result(id: string, checkId: string, passed: boolean, ranAt: string, stepId: TaskStepId | null = STEP): CheckResult {
  return {
    id: id as CheckResultId,
    checkId: checkId as DoneCheckId,
    taskId: TASK,
    stepId,
    label: checkId,
    command: 'cargo test --workspace',
    passed,
    timedOut: false,
    exitCode: passed ? 0 : 101,
    outputTail: passed ? '' : 'test export::pdf ... FAILED',
    ranAt,
    durationMs: 1500
  }
}
