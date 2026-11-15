<script setup lang="ts">
import { computed, watch } from 'vue'
import AdherenceFigure from '@/features/adherence/components/AdherenceFigure.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { ADHERENCE_HELP, shareDetail, spentBudgets, stepsLabel, visitsSummary } from '@/features/adherence/lib/adherence-figures'
import { useAdherenceStore } from '@/features/adherence/store/adherence.store'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskId } from '@/model/branded'

/**
 * How closely this task followed its plan: the steps accepted at the first pass, the ones sent
 * back, the notes that record a departure, and for a flow the visits against each budget. Only
 * the numbers, with no judgment: what they mean is the reader's to decide. It is never folded or
 * narrowed to a column; each step's passes are drawn on its row in the plan. `embedded` drops the
 * card so it can share one with the plan's progress.
 */
const props = defineProps<{ taskId: TaskId; steps: readonly TaskStep[]; embedded?: boolean }>()

const store = useAdherenceStore()
const { run } = useAsyncAction()

const adherence = computed(() => store.ofTask(props.taskId))
const planned = computed(() => props.steps.filter((step) => step.state !== 'DRAFT'))
const signature = computed(() => planned.value.map((step) => `${step.id}:${step.state}:${step.passes.length}`).join('|'))

watch(
  () => [props.taskId, signature.value],
  () => void run(() => store.loadTask(props.taskId)),
  { immediate: true }
)
</script>

<template>
  <section
    class="@container px-5 py-4"
    :class="!embedded && 'rounded-card border border-border bg-surface'"
    aria-label="Plan adherence"
    data-testid="plan-adherence"
  >
    <div class="flex flex-wrap items-baseline gap-x-3">
      <h2 class="text-title font-semibold text-text">Plan adherence</h2>
      <span class="text-caption text-text-subtle" data-testid="adherence-help">{{ ADHERENCE_HELP }}</span>
    </div>

    <p v-if="adherence && !adherence.stepsTotal" class="mt-2 text-small text-text-muted" data-testid="adherence-no-steps">
      This task has no steps, so there is no plan to measure.
    </p>

    <template v-else-if="adherence">
      <div class="mt-3 grid grid-cols-1 gap-4 @md:grid-cols-3 @md:gap-0">
        <AdherenceFigure
          class="@md:pr-5"
          label="Accepted first time"
          :value="`${adherence.acceptedFirstTime} of ${adherence.stepsDone}`"
          :detail="shareDetail(adherence.firstTimeShare)"
          testid="adherence-first-time"
        />
        <AdherenceFigure
          class="@md:border-l @md:border-border @md:px-5"
          label="Sent back"
          :value="String(adherence.sentBack)"
          :detail="`${adherence.passesTotal} ${adherence.passesTotal === 1 ? 'pass' : 'passes'} in all`"
          testid="adherence-sent-back"
        />
        <AdherenceFigure
          class="@md:border-l @md:border-border @md:pl-5"
          label="Deviations"
          :value="String(adherence.deviations)"
          :detail="adherence.deviationSteps.length ? `on ${stepsLabel(adherence.deviationSteps)}` : 'the plan was built as drawn'"
          testid="adherence-deviations"
        />
      </div>

      <details v-if="adherence.visits.length" class="group/visits mt-3 border-t border-border pt-2" data-testid="adherence-visits">
        <summary class="focus-ring flex min-h-7 cursor-pointer list-none items-center gap-2 rounded-key text-small text-text-muted hover:text-text">
          <svg class="size-2.5 shrink-0 text-text-subtle transition-transform group-open/visits:rotate-90" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
          <span class="font-medium text-text">Flow visits</span>
          <span class="text-text-subtle">{{ visitsSummary(adherence.visits) }}</span>
          <span v-if="spentBudgets(adherence.visits)" class="font-semibold text-warn" data-testid="adherence-budgets-spent">
            {{ spentBudgets(adherence.visits) }} budget{{ spentBudgets(adherence.visits) === 1 ? '' : 's' }} spent
          </span>
        </summary>
        <ul class="mt-1 grid max-h-56 grid-cols-1 gap-x-8 gap-y-0.5 overflow-y-auto @2xl:grid-cols-2">
          <li v-for="entry in adherence.visits" :key="entry.stepNumber" class="flex items-baseline gap-2 text-caption" data-testid="adherence-visit">
            <span class="w-6 shrink-0 text-right font-mono text-text-subtle">{{ entry.stepNumber }}</span>
            <span class="min-w-0 flex-1 truncate text-text-muted">{{ entry.title }}</span>
            <span class="shrink-0 font-mono text-text">{{ entry.visits }}{{ entry.maxVisits === null ? '' : ` of ${entry.maxVisits}` }}</span>
            <span v-if="entry.budgetSpent" class="shrink-0 font-semibold text-warn" data-testid="adherence-budget-spent">budget spent</span>
          </li>
        </ul>
      </details>
    </template>
  </section>
</template>
