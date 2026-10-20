import type { ProjectId, TaskId, TaskStepId, TerminalId } from '@/model/branded'

/**
 * What a terminal is opened to do: `WORK` types `/rk <anchor>`, `PLAN` types `/rk <anchor> plan`,
 * `GENERATE` types `/rk <anchor> generate "<request>"`.
 */
export type TerminalMode = 'WORK' | 'PLAN' | 'GENERATE' | 'EXPLAIN'

/** What a terminal is opened on: a task, or a project alone, which only generates a diagram. */
export type TerminalTarget = { readonly taskId: TaskId } | { readonly projectId: ProjectId }

/**
 * A live in-app terminal: a real PTY running the interactive `claude` TUI. Not persisted. The
 * task fields are null on one opened on a project alone.
 */
export interface Terminal {
  readonly id: TerminalId
  readonly projectId: ProjectId
  readonly taskId: TaskId | null
  readonly stepId: TaskStepId | null
  readonly anchors: string
  readonly workingDir: string
  readonly projectLabel: string
  readonly taskLabel: string | null
  readonly taskTitle: string | null
  readonly skipPermissions: boolean
  readonly model: string | null
  readonly effort: string | null
  readonly live: boolean
  readonly startedAt: string
  readonly lastActivityAt: string
}

/** The `{"type":"ended",...}` frame the socket sends once the PTY process exits. */
export interface TerminalEndedFrame {
  readonly type: 'ended'
  readonly exitCode: number
  readonly detail: string
}
