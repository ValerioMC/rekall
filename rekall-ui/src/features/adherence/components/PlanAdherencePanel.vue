<script setup lang="ts">
import { computed, watch } from 'vue'
import AdherenceFigure from '@/features/adherence/components/AdherenceFigure.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { ADHERENCE_HELP, passesOf, passesSummary, shareDetail, stepsLabel } from '@/features/adherence/lib/adherence-figures'
import { useAdherenceStore } from '@/features/adherence/store/adherence.store'
import type { TaskStep } from '@/features/steps/model/task-step'
import type { TaskId } from '@/model/branded'

/**
 * How closely this task followed its plan: the steps accepted at the first pass, the ones sent
 * back, the notes that record a departure, and for a flow the visits against each budget. Only
 * the numbers, with no judgment: what they mean is the reader's to decide.
 */
const props = defineProps<{ taskId: TaskId; steps: readonly TaskStep[] }>()

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
  <section class="rounded-card border border-border bg-surface px-4 py-3" aria-label="Plan adherence" data-testid="plan-adherence">
    <div class="flex flex-wrap items-baseline gap-x-3">
      <span class="eyebrow">Plan adherence</span>
      <span class="text-caption text-text-subtle" data-testid="adherence-help">{{ ADHERENCE_HELP }}</span>
    </div>

    <p v-if="adherence && !adherence.stepsTotal" class="mt-2 text-small text-text-muted" data-testid="adherence-no-steps">
      This task has no steps, so there is no plan to measure.
    </p>

    <template v-else-if="adherence">
      <div class="mt-2 grid grid-cols-1 gap-4 sm:grid-cols-3">
        <AdherenceFigure
          label="Accepted first time"
          :value="`${adherence.acceptedFirstTime} of ${adherence.stepsDone}`"
          :detail="shareDetail(adherence.firstTimeShare)"
          testid="adherence-first-time"
        />
        <AdherenceFigure
          label="Sent back"
          :value="String(adherence.sentBack)"
          :detail="`${adherence.passesTotal} ${adherence.passesTotal === 1 ? 'pass' : 'passes'} in all`"
          testid="adherence-sent-back"
        />
        <AdherenceFigure
          label="Deviations"
          :value="String(adherence.deviations)"
          :detail="adherence.deviationSteps.length ? `on ${stepsLabel(adherence.deviationSteps)}` : 'the plan was built as drawn'"
          testid="adherence-deviations"
        />
      </div>

      <p v-if="!adherence.passesTotal" class="mt-3 text-small text-text-muted" data-testid="adherence-no-passes">
        No step has been worked yet, so there are no passes to show.
      </p>

      <ol v-else class="mt-3 flex flex-col gap-1" aria-label="Passes per step">
        <li
          v-for="(step, index) in planned"
          :key="step.id"
          class="flex items-center gap-2 text-caption"
          data-testid="adherence-step"
        >
          <span class="w-6 shrink-0 text-right font-mono text-text-subtle">{{ index + 1 }}</span>
          <span class="min-w-0 flex-1 truncate text-text-muted" :title="step.title">{{ step.title }}</span>
          <span class="flex shrink-0 gap-0.5" role="img" :aria-label="passesSummary(step)">
            <span
              v-for="(pass, number) in passesOf(step)"
              :key="number"
              class="h-2 w-5 rounded-key"
              :class="pass === 'sent-back' ? 'bg-warn' : 'bg-accent'"
              data-testid="adherence-pass"
              :data-kind="pass"
            />
          </span>
          <span class="w-36 shrink-0 text-right text-text-subtle" data-testid="adherence-step-summary">{{ passesSummary(step) }}</span>
        </li>
      </ol>

      <div v-if="adherence.visits.length" class="mt-3" data-testid="adherence-visits">
        <span class="eyebrow">Flow visits</span>
        <ul class="mt-1 flex flex-col gap-0.5">
          <li v-for="entry in adherence.visits" :key="entry.stepNumber" class="flex items-baseline gap-2 text-caption" data-testid="adherence-visit">
            <span class="w-6 shrink-0 text-right font-mono text-text-subtle">{{ entry.stepNumber }}</span>
            <span class="min-w-0 flex-1 truncate text-text-muted">{{ entry.title }}</span>
            <span class="shrink-0 font-mono text-text">{{ entry.visits }}{{ entry.maxVisits === null ? '' : ` of ${entry.maxVisits}` }}</span>
            <span v-if="entry.budgetSpent" class="shrink-0 font-semibold text-warn" data-testid="adherence-budget-spent">budget spent</span>
          </li>
        </ul>
      </div>
    </template>
  </section>
</template>
