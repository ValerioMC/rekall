<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import StatusPill from '@/components/ui/StatusPill.vue'
import GraphAttachPanel from '@/features/graph/components/GraphAttachPanel.vue'
import DossierFrame from '@/features/steps/components/DossierFrame.vue'
import StepQueueRow from '@/features/steps/components/StepQueueRow.vue'
import { stepsLabel } from '@/features/adherence/lib/adherence-figures'
import { useTaskDeviations } from '@/features/deviations/composables/useTaskDeviations'
import { groupSteps, overviewTitle } from '@/features/steps/lib/plan-overview'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep } from '@/features/steps/model/task-step'

/**
 * What the pane beside the step list shows while no step is selected: the same frame a step
 * fills, holding the review queue (what waits for the reader, what is being worked, what is next,
 * what was accepted) and what else the plan has (drafts, deviations, a flow to lay). Nothing
 * moves when a step is chosen; the step takes the frame over.
 */
const props = defineProps<{
  task: Task
  /** The checklist, drafts left out. */
  steps: readonly TaskStep[]
  draftCount: number
  canLayGraph: boolean
  numberOf: (step: TaskStep) => number
}>()

const emit = defineEmits<{
  select: [step: TaskStep]
  'review-next': []
  'accept-all': []
  'show-drafts': []
  'open-deviation': [number: number]
  attached: []
}>()

const { deviations } = useTaskDeviations(computed(() => props.task.id))
const groups = computed(() => groupSteps(props.steps))
const runningStep = computed(() => groups.value.working[0] ?? null)
const title = computed(() => overviewTitle(groups.value, runningStep.value ? props.numberOf(runningStep.value) : null))
const allDone = computed(() => props.steps.length > 0 && groups.value.done.length === props.steps.length)
const readyToAcceptAll = computed(() => groups.value.waiting.length > 0 && groups.value.done.length + groups.value.waiting.length === props.steps.length)
const deviationSteps = computed(() => [...new Set(deviations.value.map((deviation) => deviation.stepNumber))].sort((first, second) => first - second))
const hasAside = computed(() => props.draftCount > 0 || deviationSteps.value.length > 0 || props.canLayGraph)

const QUEUES = computed(() => [
  { key: 'waiting', label: 'Waiting for you', steps: groups.value.waiting },
  { key: 'working', label: 'Working now', steps: groups.value.working },
  { key: 'next', label: 'Up next', steps: groups.value.upNext }
])

const hasOpenQueues = computed(() => QUEUES.value.some((queue) => queue.steps.length > 0))

const ICON_BUTTON =
  'focus-ring grid size-7 cursor-pointer place-items-center rounded-key text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text disabled:cursor-default disabled:opacity-40 disabled:hover:bg-transparent disabled:hover:text-text-subtle'
</script>

<template>
  <DossierFrame pinned aria-label="Plan overview" data-testid="plan-overview">
    <template #lead>
      <span class="grid size-full place-items-center rounded-full border border-border-strong text-text-muted">
        <svg class="size-3.5" viewBox="0 0 16 16" fill="none">
          <path d="M3 4.5h10M3 8h10M3 11.5h6" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
      </span>
    </template>
    <template #title><span data-testid="overview-title">{{ title }}</span></template>

    <template #controls>
      <button type="button" :class="ICON_BUTTON" disabled title="Previous step (K)" aria-label="Previous step">
        <svg class="size-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="M4 10l4-4 4 4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
      <button
        type="button"
        :class="ICON_BUTTON"
        :disabled="!steps.length"
        title="Next step (J)"
        aria-label="Next step"
        data-testid="overview-next"
        @click="steps[0] && emit('select', steps[0])"
      >
        <svg class="size-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="M4 6l4 4 4-4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
    </template>

    <template #facts>
      <StatusPill>No step selected</StatusPill>
      <span>{{ steps.length }} {{ steps.length === 1 ? 'step' : 'steps' }}</span>
      <span v-if="groups.done.length">{{ groups.done.length }} done</span>
      <span v-if="groups.working.length">{{ groups.working.length }} working</span>
      <span v-if="groups.upNext.length">{{ groups.upNext.length }} open</span>
    </template>

    <template #main>
      <div class="flex flex-col gap-7">
        <section
          v-for="queue in QUEUES"
          v-show="queue.steps.length"
          :key="queue.key"
          :aria-label="queue.label"
          :data-testid="`queue-${queue.key}`"
        >
          <div class="mb-1 flex items-center gap-2.5">
            <h3 class="text-small font-semibold text-text-muted">{{ queue.label }}</h3>
            <span class="h-px flex-1 bg-border" aria-hidden="true" />
            <span class="font-mono text-caption tabular-nums text-text-subtle">{{ queue.steps.length }}</span>
          </div>
          <ul>
            <StepQueueRow v-for="step in queue.steps" :key="step.id" :step="step" :number="numberOf(step)" :complete="allDone" @select="emit('select', step)" />
          </ul>
        </section>

        <details v-if="groups.done.length" class="group/done" :open="!hasOpenQueues" data-testid="queue-done">
          <summary class="focus-ring flex min-h-9 cursor-pointer list-none items-center gap-2 rounded-key text-small text-text-muted hover:text-text">
            <svg class="size-2.5 shrink-0 text-text-subtle transition-transform group-open/done:rotate-90" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            {{ groups.done.length }} accepted
          </summary>
          <ul>
            <StepQueueRow v-for="step in groups.done" :key="step.id" :step="step" :number="numberOf(step)" :complete="allDone" @select="emit('select', step)" />
          </ul>
        </details>
      </div>
    </template>

    <template #side>
      <section v-if="draftCount > 0" data-testid="overview-drafts">
        <span class="eyebrow">Drafts <span class="font-mono">{{ draftCount }}</span></span>
        <p class="mt-1 text-small text-text-muted">A draft is not part of the plan until you promote it.</p>
        <AppButton class="mt-2.5" size="sm" @click="emit('show-drafts')">Show drafts</AppButton>
      </section>

      <section v-if="deviationSteps.length" data-testid="overview-deviations">
        <span class="eyebrow">Deviations <span class="font-mono">{{ deviations.length }}</span></span>
        <p class="mt-1 text-small text-text-muted">The work left the plan on {{ stepsLabel(deviationSteps) }}.</p>
        <AppButton class="mt-2.5" size="sm" @click="emit('open-deviation', deviationSteps[0] ?? 0)">Open deviations</AppButton>
      </section>

      <GraphAttachPanel v-if="canLayGraph" :task="task" has-steps @attached="emit('attached')" />

      <p v-if="!hasAside" class="text-small text-text-subtle" data-testid="overview-aside-empty">
        Drafts, deviations and a flow to lay show here when the task has them.
      </p>
    </template>

    <template #dock>
      <p class="min-w-0 flex-1 text-small text-text-muted">Select a step to read it. J and K move between steps, Esc comes back here.</p>
      <div class="flex items-center gap-2">
        <AppButton v-if="groups.waiting.length" :variant="readyToAcceptAll ? 'secondary' : 'primary'" data-testid="overview-review-next" @click="emit('review-next')">
          Review next
        </AppButton>
        <AppButton
          v-if="readyToAcceptAll"
          variant="primary"
          title="Accept every claimed step"
          data-testid="steps-accept-task"
          @click="emit('accept-all')"
        >
          Accept all
        </AppButton>
      </div>
    </template>
  </DossierFrame>
</template>
