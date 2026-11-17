<script setup lang="ts">
import { storeToRefs } from 'pinia'
import FormPanel from '@/components/ui/FormPanel.vue'
import FormRow from '@/components/ui/FormRow.vue'
import AppInput from '@/components/ui/AppInput.vue'
import { USAGE_WARNING_RANGE, WORKDAY_GOAL_RANGE } from '@/common/config/ui-preferences'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'

const preferences = usePreferencesStore()
const { workdayGoalHours, usageWarningPercent } = storeToRefs(preferences)
</script>

<template>
  <FormPanel title="Time and usage" aria-label="Time and usage" data-testid="time-usage-section">
    <FormRow label="Workday goal, in hours" label-for="workday-goal" :hint="`The dashed line on the time report. From ${WORKDAY_GOAL_RANGE.min} to ${WORKDAY_GOAL_RANGE.max}.`">
      <div class="w-24">
        <AppInput id="workday-goal" type="number" mono :model-value="workdayGoalHours" @update:model-value="preferences.setWorkdayGoal(Number($event))" />
      </div>
    </FormRow>
    <FormRow label="Warn when Claude usage reaches, in percent" label-for="usage-warning" :hint="`The usage meter in the header turns amber from this point. From ${USAGE_WARNING_RANGE.min} to ${USAGE_WARNING_RANGE.max}.`">
      <div class="w-24">
        <AppInput id="usage-warning" type="number" mono :model-value="usageWarningPercent" @update:model-value="preferences.setUsageWarning(Number($event))" />
      </div>
    </FormRow>
  </FormPanel>
</template>
