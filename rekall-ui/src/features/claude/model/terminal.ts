import type { ConnectionKind } from '@/features/connection/model/connection'
import type { ConnectionId, ProjectId, TaskId, TaskStepId, TerminalId } from '@/model/branded'

/**
 * What a terminal is opened to do: `WORK` types `/rk <anchor>`, `PLAN` types `/rk <anchor> plan`,
 * `PLAN_FLOW` types `/rk <anchor> plan flow`, `GENERATE` types `/rk <anchor> generate "<request>"`.
 */
export type TerminalMode = 'WORK' | 'PLAN' | 'PLAN_FLOW' | 'GENERATE' | 'EXPLAIN'

/** A planning terminal: a checklist of drafts, or a flow, drafts joined by their routes. */
export type PlanMode = Extract<TerminalMode, 'PLAN' | 'PLAN_FLOW'>

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
  /** What the terminal's project reaches outside Rekall; never a credential. */
  readonly connections: readonly TerminalConnection[]
}

/** A connection a terminal's helpers can reach, or cannot when no credential is stored. */
export interface TerminalConnection {
  readonly connectionId: ConnectionId
  readonly label: string
  readonly kind: ConnectionKind
  readonly credentialSet: boolean
}

/** The `{"type":"ended",...}` frame the socket sends once the PTY process exits. */
export interface TerminalEndedFrame {
  readonly type: 'ended'
  readonly exitCode: number
  readonly detail: string
}
