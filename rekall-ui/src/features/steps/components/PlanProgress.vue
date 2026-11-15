<script setup lang="ts">
import { computed } from 'vue'
import PlanTube from '@/components/ui/PlanTube.vue'
import { planSummary, type PlanSummaryCounts } from '@/features/steps/lib/plan-summary'

/** The plan as one tube and a line saying what waits on the reader or is happening, and how many steps are still drafts. */
const props = defineProps<PlanSummaryCounts & { draftCount: number }>()

defineEmits<{ 'show-drafts': [] }>()

const summary = computed(() => planSummary(props))
const running = computed(() => props.runningNumber !== null)
</script>

<template>
  <div class="flex min-w-0 flex-col justify-center gap-2.5 px-5 py-4" data-testid="plan-progress">
    <PlanTube :done="done" :claimed="claimed" :total="total" :live="running" size="lg" />
    <p class="text-small text-text-muted" data-testid="plan-summary-line">
      <b v-if="summary.lead" class="font-semibold" :class="summary.waitsForReader ? 'text-accent' : 'text-session-strong'">
        {{ summary.lead }}
      </b>
      {{ summary.tally }}
      <button
        v-if="draftCount > 0"
        type="button"
        class="focus-ring ml-2 rounded-key text-text-subtle transition-colors hover:text-text"
        data-testid="steps-draft-count"
        @click="$emit('show-drafts')"
      >
        {{ draftCount }} in draft
      </button>
    </p>
  </div>
</template>
