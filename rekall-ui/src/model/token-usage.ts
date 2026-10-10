import type { TaskId, TaskStepId } from '@/model/branded'

/** Every session's tokens on one step, or on its task outside any step, as the server sums them. */
export interface TokenTally {
  readonly taskId: TaskId
  readonly stepId: TaskStepId | null
  readonly inputTokens: number
  readonly outputTokens: number
  readonly cacheCreationTokens: number
  readonly cacheReadTokens: number
}

/** A task's tallies after a transcript read changed them: all of them, replacing what was held. */
export interface TokenUsageStreamEvent {
  readonly taskId: TaskId
  readonly tallies: readonly TokenTally[]
}

/**
 * Tokens summed over a step, a task or a project. `generated` is what the model wrote, the figure
 * the console shows; the rest is what it read, kept for the breakdown.
 */
export interface TokenTotals {
  readonly generated: number
  readonly input: number
  readonly cacheWrite: number
  readonly cacheRead: number
}

export const NO_TOKENS: TokenTotals = { generated: 0, input: 0, cacheWrite: 0, cacheRead: 0 }
