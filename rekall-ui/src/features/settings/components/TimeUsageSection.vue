<script setup lang="ts">
import { storeToRefs } from 'pinia'
import AppInput from '@/components/ui/AppInput.vue'
import { USAGE_WARNING_RANGE, WORKDAY_GOAL_RANGE } from '@/common/config/ui-preferences'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'

const preferences = usePreferencesStore()
const { workdayGoalHours, usageWarningPercent } = storeToRefs(preferences)
</script>

<template>
  <section class="flex flex-col gap-6" aria-label="Time and usage" data-testid="time-usage-section">
    <h2 class="text-heading font-medium text-text">Time and usage</h2>
    <div class="flex flex-col gap-2">
      <label class="text-body font-medium text-text" for="workday-goal">Workday goal, in hours</label>
      <p class="text-small text-text-muted">The dashed line on the time report. From {{ WORKDAY_GOAL_RANGE.min }} to {{ WORKDAY_GOAL_RANGE.max }}.</p>
      <div class="w-24">
        <AppInput
          id="workday-goal"
          type="number"
          mono
          :model-value="workdayGoalHours"
          @update:model-value="preferences.setWorkdayGoal(Number($event))"
        />
      </div>
    </div>
    <div class="flex flex-col gap-2">
      <label class="text-body font-medium text-text" for="usage-warning">Warn when Claude usage reaches, in percent</label>
      <p class="text-small text-text-muted">The usage meter in the header turns amber from this point. From {{ USAGE_WARNING_RANGE.min }} to {{ USAGE_WARNING_RANGE.max }}.</p>
      <div class="w-24">
        <AppInput
          id="usage-warning"
          type="number"
          mono
          :model-value="usageWarningPercent"
          @update:model-value="preferences.setUsageWarning(Number($event))"
        />
      </div>
    </div>
  </section>
</template>
