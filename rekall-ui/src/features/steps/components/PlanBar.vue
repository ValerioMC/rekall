<script setup lang="ts">
import { computed, ref, toRef } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import PlanTube from '@/components/ui/PlanTube.vue'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'
import { useDismissable } from '@/composables/useDismissable'
import PlanAdherenceFigures from '@/features/adherence/components/PlanAdherenceFigures.vue'
import { useTaskAdherence } from '@/features/adherence/composables/useTaskAdherence'
import { spentBudgets } from '@/features/adherence/lib/adherence-figures'
import { planSummary, type PlanSummaryCounts } from '@/features/steps/lib/plan-summary'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskId } from '@/model/branded'

/**
 * One row over the plan, whichever view is open: the tube and who has to act, the adherence
 * figures as a line that opens the full panel over the workspace, and the keys that narrow
 * the steps (hide the done ones, jump to the next claim, list or flow).
 */
const props = defineProps<
  PlanSummaryCounts & {
    taskId: TaskId
    steps: readonly TaskStep[]
    draftCount: number
    canShowFlow: boolean
    listKeys: boolean
  }
>()

const hideDone = defineModel<boolean>('hideDone', { required: true })
const view = defineModel<'list' | 'flow'>('view', { required: true })

defineEmits<{ 'show-drafts': []; 'review-next': [] }>()

const VIEWS: readonly SegmentOption<'list' | 'flow'>[] = [
  { value: 'list', label: 'List' },
  { value: 'flow', label: 'Flow' }
]

const summary = computed(() => planSummary(props))
const running = computed(() => props.runningNumber !== null)
const adherence = useTaskAdherence(toRef(props, 'taskId'), toRef(props, 'steps'))
const spent = computed(() => (adherence.value ? spentBudgets(adherence.value.visits) : 0))
const measured = computed(() => adherence.value !== null && adherence.value.stepsTotal > 0)

const open = ref(false)
const host = ref<HTMLElement | null>(null)
useDismissable(host, open)
</script>

<template>
  <div class="relative z-10 flex min-h-11 shrink-0 flex-wrap items-center gap-x-4 gap-y-1 border-b border-border px-6 py-1.5" data-testid="plan-bar">
    <PlanTube class="w-40 shrink-0" :done="done" :claimed="claimed" :total="total" :live="running" size="md" />
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

    <div v-if="measured && adherence" ref="host" class="relative">
      <button
        type="button"
        class="focus-ring flex h-7 cursor-pointer items-center gap-3.5 rounded-control px-2.5 text-small text-text-muted transition-colors hover:bg-surface-raised hover:text-text"
        :class="open && 'bg-surface-raised text-text'"
        :aria-expanded="open"
        aria-controls="plan-adherence-panel"
        data-testid="adherence-figures"
        @click="open = !open"
      >
        <span><b class="font-mono font-medium text-text">{{ adherence.acceptedFirstTime }}/{{ adherence.stepsDone }}</b> first time</span>
        <span><b class="font-mono font-medium text-text">{{ adherence.sentBack }}</b> sent back</span>
        <span><b class="font-mono font-medium text-text">{{ adherence.deviations }}</b> {{ adherence.deviations === 1 ? 'deviation' : 'deviations' }}</span>
        <span v-if="spent" class="text-warn" data-testid="adherence-budgets-spent-chip">
          <b class="font-mono font-medium">{{ spent }}</b> {{ spent === 1 ? 'budget' : 'budgets' }} spent
        </span>
        <svg class="size-2.5 text-text-subtle transition-transform" :class="open && 'rotate-90'" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
      <div
        v-if="open"
        id="plan-adherence-panel"
        class="absolute left-0 top-full z-(--z-overlay) mt-1.5 w-[640px] max-w-[calc(100vw-3rem)] rounded-card border border-border-strong bg-surface px-5 py-4 shadow-(--shadow-lift)"
      >
        <PlanAdherenceFigures :adherence="adherence" />
      </div>
    </div>

    <span class="flex-1" />
    <AppButton
      v-if="listKeys && (done > 0 || claimed > 0)"
      variant="quiet"
      size="sm"
      :aria-pressed="hideDone"
      data-testid="toggle-hide-done"
      @click="hideDone = !hideDone"
    >
      {{ hideDone ? 'Show done' : 'Hide done' }}
    </AppButton>
    <AppButton v-if="claimed > 0" size="sm" data-testid="plan-review-next" @click="$emit('review-next')">Review next</AppButton>
    <SegmentedControl v-if="canShowFlow" v-model="view" :options="VIEWS" label="Show the steps as" size="sm" />
  </div>
</template>
