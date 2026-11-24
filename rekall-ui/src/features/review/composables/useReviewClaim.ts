import { computed, type ComputedRef } from 'vue'
import { failedRuns, latestRunsOfStep } from '@/features/check/lib/check-results'
import { useChecksStore } from '@/features/check/store/checks.store'
import { stepNumberOf } from '@/features/deviations/model/deviation'
import { useDeviationsStore } from '@/features/deviations/store/deviations.store'
import { claimFacts, type ClaimFact } from '@/features/review/lib/review-claim-facts'
import { reviewItemLine } from '@/features/review/lib/review-item-line'
import { useConsoleStore } from '@/stores/console.store'
import type { Task } from '@/features/catalog/model/catalog'
import type { ReviewItem } from '@/features/catalog/model/review'
import type { CheckResult } from '@/features/check/model/check'
import type { CommitReference } from '@/features/commits/model/commitReference'
import type { TaskDeviation } from '@/features/deviations/model/deviation'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { Wrapup } from '@/features/wrapup/model/wrapup'

export interface ReviewClaim {
  readonly task: ComputedRef<Task | null>
  readonly step: ComputedRef<TaskStep | null>
  readonly line: ComputedRef<string>
  readonly runs: ComputedRef<readonly CheckResult[]>
  readonly failedRuns: ComputedRef<readonly CheckResult[]>
  readonly commits: ComputedRef<readonly CommitReference[]>
  readonly deviation: ComputedRef<TaskDeviation | null>
  readonly wrapup: ComputedRef<Wrapup | null>
  readonly facts: ComputedRef<readonly ClaimFact[]>
}

/** What the console knows about one queue item: its task and step, and the evidence found on it. */
export function useReviewClaim(item: () => ReviewItem): ReviewClaim {
  const store = useConsoleStore()
  const checks = useChecksStore()
  const deviations = useDeviationsStore()

  const task = computed(() => store.tasks.find((candidate) => candidate.id === item().taskId) ?? null)
  const taskSteps = computed(() => store.steps.filter((candidate) => candidate.taskId === item().taskId).sort((a, b) => a.position - b.position))
  const step = computed(() => taskSteps.value.find((candidate) => candidate.id === item().stepId) ?? null)
  const place = computed(() => {
    const number = step.value ? stepNumberOf(step.value, taskSteps.value) : null
    return number === null ? null : { number, total: taskSteps.value.filter((candidate) => candidate.state !== 'DRAFT').length }
  })
  const line = computed(() => (task.value ? reviewItemLine(task.value, place.value) : ''))
  const runs = computed(() => (step.value ? latestRunsOfStep(checks.resultsOf(item().taskId), step.value.id) : []))
  const commits = computed(() => store.commitReferences.filter((reference) => step.value !== null && reference.stepId === step.value.id))
  const deviation = computed(() => deviations.of(item().taskId).find((candidate) => candidate.stepNumber === place.value?.number) ?? null)
  const wrapup = computed(() => (step.value ? null : (store.wrapups.find((candidate) => candidate.taskId === item().taskId) ?? null)))
  const facts = computed(() => claimFacts({ runs: runs.value, commitCount: commits.value.length, hasDeviation: deviation.value !== null }))

  return { task, step, line, runs, failedRuns: computed(() => failedRuns(runs.value)), commits, deviation, wrapup, facts }
}
