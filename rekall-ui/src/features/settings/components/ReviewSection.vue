<script setup lang="ts">
import { storeToRefs } from 'pinia'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'

const { reviewAutoAdvance, reviewConfirmSendBack } = storeToRefs(usePreferencesStore())
</script>

<template>
  <section class="flex flex-col gap-5" aria-label="Review" data-testid="review-section">
    <h2 class="text-heading font-medium text-text">Review</h2>
    <div class="flex flex-col gap-1.5">
      <AppCheckbox v-model="reviewAutoAdvance" label="Open the next item after a decision" />
      <p class="pl-[26px] text-small text-text-muted">Off leaves nothing selected on Review until you pick the next item.</p>
    </div>
    <div class="flex flex-col gap-1.5">
      <AppCheckbox v-model="reviewConfirmSendBack" label="Confirm before sending work back" />
      <p class="pl-[26px] text-small text-text-muted">Asks once, wherever a step or a task is sent back: Work, the task page and Review.</p>
    </div>
  </section>
</template>
