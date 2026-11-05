import type { ConnectionKind } from '@/features/connection/model/connection'
import type { ConnectionId, ProjectId, TaskId, TaskStepId, TerminalId } from '@/model/branded'

/**
 * What a terminal is opened to do: `WORK` types `/rk <anchor>`, `PLAN` types `/rk <anchor> plan`,
 * `PLAN_FLOW` types `/rk <anchor> plan flow`, `REPLAN` types `/rk <anchor> replan`, `REVISE` types `/rk <anchor> revise`, `GENERATE` types `/rk <anchor> generate "<request>"`,
 * `INVENTORY` types `/rk <anchor> inventory`, `ASSESS` types `/rk <anchor> assess advisory:<id>`,
 * `ASSESS_ALL` types `/rk <anchor> assess` to judge every advisory not assessed yet, `AUTOCONFIGURE`
 * types `/rk <anchor> autoconfigure` to learn how the project takes a contribution.
 */
export type TerminalMode = 'WORK' | 'PLAN' | 'PLAN_FLOW' | 'REPLAN' | 'REVISE' | 'GENERATE' | 'EXPLAIN' | 'INVENTORY' | 'ASSESS' | 'ASSESS_ALL' | 'AUTOCONFIGURE'

/**
 * A terminal that acts on a task's plan or thread: a checklist of drafts, a flow, drafts joined by
 * their routes, a replan that swaps the task's draft and open steps for a new plan, or a revise
 * that evaluates the pending entries of its thread.
 */
export type PlanMode = Extract<TerminalMode, 'PLAN' | 'PLAN_FLOW' | 'REPLAN' | 'REVISE'>

/** What a terminal is opened on: a task, or a project alone, which draws a diagram, inventories, assesses or autoconfigures. */
export type TerminalTarget = { readonly taskId: TaskId } | { readonly projectId: ProjectId }

/** How a session runs `claude`: the interactive TUI in a PTY, or headless over stream-json with its own window. */
export type SessionKind = 'TERMINAL' | 'CHAT'

/**
 * A live in-app session: a real PTY running the interactive `claude` TUI, or a chat session
 * running it headless. Not persisted. The task fields are null on one opened on a project alone.
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
  readonly mode: TerminalMode
  readonly skipPermissions: boolean
  readonly model: string | null
  readonly effort: string | null
  readonly kind: SessionKind
  readonly live: boolean
  /** A chat session stopped on a permission prompt nobody has answered; always false for a terminal. */
  readonly awaitingPermission: boolean
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
