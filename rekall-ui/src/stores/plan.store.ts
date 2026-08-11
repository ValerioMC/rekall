import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { applyPlan, fetchPlan, planWithAnswers } from '@/api/plan.api'
import type { ApplyResult, Plan, PlanAnswers } from '@/model/plan'

export const usePlanStore = defineStore('plan', () => {
  const plan = ref<Plan | null>(null)
  const isLoading = ref(false)
  const isApplying = ref(false)

  const backfills = ref<Record<string, string>>({})
  const confirmations = ref<Record<string, boolean>>({})

  const pendingCount = computed(() => plan.value?.statements.length ?? 0)

  const answers = computed<PlanAnswers>(() => ({
    backfillDefaults: Object.fromEntries(
      Object.entries(backfills.value).filter(([, value]) => value.trim().length > 0)
    ),
    confirmedDrops: Object.entries(confirmations.value)
      .filter(([, confirmed]) => confirmed)
      .map(([key]) => key)
  }))

  /** Loads the plan as it stands, with whatever answers have been given so far. */
  async function refresh(): Promise<void> {
    isLoading.value = true
    try {
      const current = answers.value
      plan.value =
        current.confirmedDrops.length === 0 && Object.keys(current.backfillDefaults).length === 0
          ? await fetchPlan()
          : await planWithAnswers(current)
    } finally {
      isLoading.value = false
    }
  }

  async function apply(): Promise<ApplyResult> {
    isApplying.value = true
    try {
      const result = await applyPlan(answers.value)
      backfills.value = {}
      confirmations.value = {}
      await refresh()
      return result
    } finally {
      isApplying.value = false
    }
  }

  function setBackfill(key: string, value: string): void {
    backfills.value = { ...backfills.value, [key]: value }
  }

  function setConfirmation(key: string, confirmed: boolean): void {
    confirmations.value = { ...confirmations.value, [key]: confirmed }
  }

  return {
    plan,
    isLoading,
    isApplying,
    backfills,
    confirmations,
    pendingCount,
    answers,
    refresh,
    apply,
    setBackfill,
    setConfirmation
  }
})
