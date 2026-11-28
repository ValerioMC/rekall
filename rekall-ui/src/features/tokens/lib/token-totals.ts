import type { ProjectId, TaskId, TaskStepId } from '@/model/branded'
import type { Task } from '@/features/catalog/model/catalog'
import { SESSION_PROVIDERS } from '@/features/claude/model/session-provider'
import { NO_TOKENS, type ProviderTokenTally, type TokenDay, type TokenTally, type TokenTotals } from '@/features/tokens/model/token-usage'

type TokenSpend = Pick<TokenTally, 'outputTokens' | 'inputTokens' | 'cacheCreationTokens' | 'cacheReadTokens'>

export function addTokenSpend(totals: TokenTotals, spend: TokenSpend): TokenTotals {
  return {
    generated: totals.generated + spend.outputTokens,
    input: totals.input + spend.inputTokens,
    cacheWrite: totals.cacheWrite + spend.cacheCreationTokens,
    cacheRead: totals.cacheRead + spend.cacheReadTokens
  }
}

function summed(tallies: readonly TokenTally[]): TokenTotals {
  return tallies.reduce(addTokenSpend, NO_TOKENS)
}

export function tokensByProvider(tallies: readonly TokenTally[]): ProviderTokenTally[] {
  return SESSION_PROVIDERS.flatMap((provider) => {
    const fromProvider = tallies.filter((tally) => (tally.provider ?? 'CLAUDE') === provider)
    return fromProvider.length ? [{ provider, totals: summed(fromProvider) }] : []
  })
}

export function stepTokens(tallies: readonly TokenTally[], stepId: TaskStepId): TokenTotals {
  return summed(tallies.filter((tally) => tally.stepId === stepId))
}

/** A task's own work and every one of its steps'. */
export function taskTokens(tallies: readonly TokenTally[], taskId: TaskId): TokenTotals {
  return summed(tallies.filter((tally) => tally.taskId === taskId))
}

/** The sum of the project's tasks. */
export function projectTokens(
  tallies: readonly TokenTally[],
  tasks: readonly Task[],
  projectId: ProjectId
): TokenTotals {
  const taskIds = new Set(tasks.filter((task) => task.projectId === projectId).map((task) => task.id))
  return summed(tallies.filter((tally) => taskIds.has(tally.taskId)))
}

/** The tallies held, with one task's swapped for the ones a stream event carries. */
export function withTaskTallies(
  tallies: readonly TokenTally[],
  taskId: TaskId,
  replacement: readonly TokenTally[]
): TokenTally[] {
  return [...tallies.filter((tally) => tally.taskId !== taskId), ...replacement]
}

/** The days held, with one task's swapped for the ones a stream event carries. */
export function withTaskDays(days: readonly TokenDay[], taskId: TaskId, replacement: readonly TokenDay[]): TokenDay[] {
  return [...days.filter((day) => day.taskId !== taskId), ...replacement]
}
