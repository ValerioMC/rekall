import type { CheckResultId, DoneCheckId, ProjectId, TaskId, TaskStepId } from '@/model/branded'

/**
 * A command that has to pass before a claim is accepted. It belongs to the project, or to one
 * step when `stepId` is set.
 */
export interface DoneCheck {
  readonly id: DoneCheckId
  readonly projectId: ProjectId
  readonly stepId: TaskStepId | null
  readonly label: string
  readonly command: string
  /** Relative to the project's folder; null runs in the folder itself. */
  readonly folder: string | null
  readonly timeoutSeconds: number
  readonly position: number
  readonly createdAt: string
  readonly updatedAt: string
}

/** Every field may be left out on an edit; a blank folder clears it. */
export interface CheckInput {
  readonly label?: string
  readonly command?: string
  readonly folder?: string
  readonly timeoutSeconds?: number
}

/** One kept run of a check, with the check's label and command as they were when it ran. */
export interface CheckResult {
  readonly id: CheckResultId
  readonly checkId: DoneCheckId
  readonly taskId: TaskId
  readonly stepId: TaskStepId | null
  readonly label: string
  readonly command: string
  readonly passed: boolean
  readonly timedOut: boolean
  readonly exitCode: number | null
  readonly outputTail: string
  readonly ranAt: string
  readonly durationMs: number
}

export interface ProjectCheckStreamEvent {
  readonly projectId: ProjectId
}

export interface TaskCheckResultStreamEvent {
  readonly taskId: TaskId
}
