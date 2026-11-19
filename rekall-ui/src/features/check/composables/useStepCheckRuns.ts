import { computed, type ComputedRef, type Ref } from 'vue'
import { failedRuns, latestRunsOfStep } from '@/features/check/lib/check-results'
import { useChecksStore } from '@/features/check/store/checks.store'
import type { CheckResult } from '@/features/check/model/check'
import type { TaskStep } from '@/features/steps/model/task-step'

export interface StepCheckRuns {
  readonly runs: ComputedRef<CheckResult[]>
  readonly failures: ComputedRef<number>
}

export function useStepCheckRuns(step: Ref<TaskStep>): StepCheckRuns {
  const store = useChecksStore()
  const runs = computed(() => latestRunsOfStep(store.resultsOf(step.value.taskId), step.value.id))
  const failures = computed(() => failedRuns(runs.value).length)
  return { runs, failures }
}
