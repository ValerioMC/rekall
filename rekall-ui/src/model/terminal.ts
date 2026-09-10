import type { TaskId, TaskStepId, TerminalId } from '@/model/branded'

/** A live in-app terminal: a real PTY running the interactive `claude` TUI. Not persisted. */
export interface Terminal {
  readonly id: TerminalId
  readonly taskId: TaskId
  readonly stepId: TaskStepId | null
  readonly anchors: string
  readonly workingDir: string
  readonly projectLabel: string
  readonly taskLabel: string
  readonly taskTitle: string
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
