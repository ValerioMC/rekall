import type { CheckResult } from '@/features/check/model/check'
import type { TaskStepId } from '@/model/branded'

/** The last run of each check on a step, in the order they ran. */
export function latestRunsOfStep(results: readonly CheckResult[], stepId: TaskStepId): CheckResult[] {
  const latest = new Map<string, CheckResult>()
  for (const result of results) {
    if (result.stepId !== stepId) continue
    const known = latest.get(result.checkId)
    if (!known || Date.parse(result.ranAt) > Date.parse(known.ranAt)) latest.set(result.checkId, result)
  }
  return [...latest.values()].sort((first, second) => Date.parse(first.ranAt) - Date.parse(second.ranAt))
}

export function failedRuns(runs: readonly CheckResult[]): CheckResult[] {
  return runs.filter((run) => !run.passed)
}

export function runDuration(durationMs: number): string {
  if (durationMs < 1000) return `${durationMs} ms`
  const seconds = durationMs / 1000
  if (seconds < 60) return `${seconds.toFixed(1)} s`
  return `${Math.floor(seconds / 60)} min ${Math.round(seconds % 60)} s`
}

export function runOutcome(run: CheckResult): string {
  if (run.passed) return 'Passed'
  if (run.timedOut) return 'Timed out'
  return `Failed, exit ${run.exitCode}`
}
