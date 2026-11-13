import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import {
  USAGE_WARNING_RANGE,
  WORKDAY_GOAL_RANGE,
  loadUiPreferences,
  saveUiPreferences,
  withinRange
} from '@/common/config/ui-preferences'
import type { Motion, Scale } from '@/common/config/ui-preferences'

/** The browser-side choices of the Settings screen: motion, text scale, how Review asks, the workday and the usage warning. */
export const usePreferencesStore = defineStore('preferences', () => {
  const remembered = loadUiPreferences()

  const motion = ref<Motion>(remembered.motion)
  const scale = ref<Scale>(remembered.scale)
  const reviewAutoAdvance = ref(remembered.reviewAutoAdvance)
  const reviewConfirmSendBack = ref(remembered.reviewConfirmSendBack)
  const workdayGoalHours = ref(remembered.workdayGoalHours)
  const usageWarningPercent = ref(remembered.usageWarningPercent)

  function setWorkdayGoal(hours: number): void {
    workdayGoalHours.value = withinRange(hours, WORKDAY_GOAL_RANGE, workdayGoalHours.value)
  }

  function setUsageWarning(percent: number): void {
    usageWarningPercent.value = withinRange(percent, USAGE_WARNING_RANGE, usageWarningPercent.value)
  }

  watch([motion, scale, reviewAutoAdvance, reviewConfirmSendBack, workdayGoalHours, usageWarningPercent], () =>
    saveUiPreferences({
      motion: motion.value,
      scale: scale.value,
      reviewAutoAdvance: reviewAutoAdvance.value,
      reviewConfirmSendBack: reviewConfirmSendBack.value,
      workdayGoalHours: workdayGoalHours.value,
      usageWarningPercent: usageWarningPercent.value
    })
  )

  return { motion, scale, reviewAutoAdvance, reviewConfirmSendBack, workdayGoalHours, usageWarningPercent, setWorkdayGoal, setUsageWarning }
})
