<script setup lang="ts">
import { storeToRefs } from 'pinia'
import FormPanel from '@/components/ui/FormPanel.vue'
import FormRow from '@/components/ui/FormRow.vue'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'

const { reviewAutoAdvance, reviewConfirmSendBack } = storeToRefs(usePreferencesStore())
</script>

<template>
  <FormPanel title="Review" aria-label="Review" data-testid="review-section">
    <FormRow label="Open the next item after a decision" hint="Off leaves nothing selected on Review until you pick the next item.">
      <AppCheckbox v-model="reviewAutoAdvance" label="On" />
    </FormRow>
    <FormRow label="Confirm before sending work back" hint="Asks once, wherever a step or a task is sent back: a project's task list, the task page and Review.">
      <AppCheckbox v-model="reviewConfirmSendBack" label="On" />
    </FormRow>
  </FormPanel>
</template>
