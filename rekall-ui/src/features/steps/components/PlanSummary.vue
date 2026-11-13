<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import PlanTube from '@/components/ui/PlanTube.vue'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'
import { planSummary, type PlanSummaryCounts } from '@/features/steps/lib/plan-summary'

/**
 * The line at the head of the plan: the tube, what waits on the reader or is happening, and the
 * keys that narrow the list (hide the done steps, jump to the next claim, list or flow).
 */
const props = defineProps<PlanSummaryCounts & { draftCount: number; canShowFlow: boolean }>()

const hideDone = defineModel<boolean>('hideDone', { required: true })
const view = defineModel<'list' | 'flow'>('view', { required: true })

defineEmits<{ 'review-next': []; 'show-drafts': []; 'accept-all': [] }>()

const VIEWS: readonly SegmentOption<'list' | 'flow'>[] = [
  { value: 'list', label: 'List' },
  { value: 'flow', label: 'Flow' }
]

const summary = computed(() => planSummary(props))
const running = computed(() => props.runningNumber !== null)
const readyToAcceptAll = computed(() => props.claimed > 0 && props.done + props.claimed === props.total)
</script>

<template>
  <div class="mb-4 flex flex-wrap items-center gap-x-4 gap-y-2" data-testid="plan-summary">
    <div class="flex min-w-[min(100%,16rem)] flex-1 flex-col gap-1.5">
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

    <div class="flex shrink-0 items-center gap-2">
      <SegmentedControl v-if="canShowFlow" v-model="view" :options="VIEWS" label="Show the steps as" />
      <AppButton
        v-if="done > 0 || claimed > 0"
        variant="quiet"
        size="sm"
        :aria-pressed="hideDone"
        data-testid="toggle-hide-done"
        @click="hideDone = !hideDone"
      >
        {{ hideDone ? 'Show done' : 'Hide done' }}
      </AppButton>
      <AppButton v-if="claimed > 0" size="sm" data-testid="plan-review-next" @click="$emit('review-next')">
        Review next
      </AppButton>
      <AppButton
        v-if="readyToAcceptAll"
        variant="primary"
        size="sm"
        title="Accept every claimed step"
        data-testid="steps-accept-task"
        @click="$emit('accept-all')"
      >
        Accept all
      </AppButton>
    </div>
  </div>
</template>
