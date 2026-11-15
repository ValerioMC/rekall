<script setup lang="ts">
import { computed, toRef } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import StatusPill, { type StatusPillTone } from '@/components/ui/StatusPill.vue'
import DossierFrame from '@/features/steps/components/DossierFrame.vue'
import StepDecisionKeys from '@/features/steps/components/StepDecisionKeys.vue'
import StepDetailBody from '@/features/steps/components/StepDetailBody.vue'
import StepEvidence from '@/features/steps/components/StepEvidence.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import { useTaskDeviations } from '@/features/deviations/composables/useTaskDeviations'
import { useStepCheckRuns } from '@/features/check/composables/useStepCheckRuns'
import { checksLine as checksLineOf } from '@/features/check/lib/check-results'
import { useConsoleStore } from '@/stores/console.store'
import { relativeTime } from '@/common/format/relative-time'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'

/**
 * The selected step as a decision: a header that names the step and says what was found on it, a
 * body that reads what it says beside the evidence for it, and a dock holding the keys a person
 * decides with. Pinned, the header and dock stay while the body scrolls, so a long detail is read
 * with the decision in reach. Only a person marks a step done.
 */
const props = defineProps<{
  step: TaskStep
  task: Task
  number: number
  total: number
  reopenArmed: boolean
  /** The header and dock stay and the body scrolls; unset, the panel grows with its text inside a page that scrolls. */
  pinned?: boolean
  /** Whether the step list is hidden so the panel has the whole tab. Unset where there is no list to hide. */
  focused?: boolean
  /** The steps either side of this one, to step through. Unset where there is no list to step through. */
  neighbours?: { readonly previous: TaskStep | null; readonly next: TaskStep | null }
}>()

defineEmits<{
  accept: []
  'send-back': []
  reopen: []
  'mark-done': []
  select: [step: TaskStep]
  overview: []
  'toggle-focus': []
  'open-deviation': [number: number]
}>()

const store = useConsoleStore()
const { deviations } = useTaskDeviations(computed(() => props.task.id))
const { runs, failures } = useStepCheckRuns(toRef(props, 'step'))
const hasDeviation = computed(() => deviations.value.some((deviation) => deviation.stepNumber === props.number))
const commitCount = computed(() => store.stepCommitReferences(props.step.id).length)
const passNumber = computed(() => props.step.passes.length + 1)

const STATE_WORD: Readonly<Record<TaskStepState, string>> = {
  DRAFT: 'Draft',
  OPEN: 'Open',
  RUNNING: 'Working',
  CLAIMED: 'Claimed',
  DONE: 'Done'
}

const STATE_TONE: Readonly<Record<TaskStepState, StatusPillTone>> = {
  DRAFT: 'neutral',
  OPEN: 'neutral',
  RUNNING: 'running',
  CLAIMED: 'running',
  DONE: 'done'
}

const DOCK_NOTE: Readonly<Record<TaskStepState, string>> = {
  DRAFT: '',
  OPEN: 'Claude gets this detail while the step is open.',
  RUNNING: 'A session is working on this step. Log its commit as it lands, or mark it done yourself.',
  CLAIMED: 'Accept it to tick the step, or send it back for another pass.',
  DONE: 'Accepted. Reopening drops it back to open and clears the review it passed.'
}

const stateMoment = computed(() => {
  const { step } = props
  const moment = step.state === 'CLAIMED' ? step.claimedAt : step.state === 'DONE' ? step.doneAt : null
  return moment ? relativeTime(moment) : null
})

const checksLine = computed(() => checksLineOf(runs.value))

const ICON_BUTTON =
  'focus-ring grid size-7 cursor-pointer place-items-center rounded-key text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text disabled:cursor-default disabled:opacity-40 disabled:hover:bg-transparent disabled:hover:text-text-subtle'
</script>

<template>
  <DossierFrame :pinned="pinned" :focused="focused" aria-label="Decision" data-testid="decision-panel" :data-step-state="step.state">
    <template #lead><StepSeal :state="step.state" /></template>
    <template #title><span data-testid="decision-title">{{ step.title }}</span></template>

    <template #controls>
      <AppButton v-if="focused !== undefined" variant="quiet" size="sm" title="Back to the overview (Esc)" data-testid="decision-overview" @click="$emit('overview')">
        Overview
        <kbd class="font-mono text-micro text-text-subtle">Esc</kbd>
      </AppButton>
      <template v-if="neighbours">
        <button
          type="button"
          :class="ICON_BUTTON"
          :disabled="!neighbours.previous"
          title="Previous step (K)"
          aria-label="Previous step"
          data-testid="decision-previous"
          @click="neighbours.previous && $emit('select', neighbours.previous)"
        >
          <svg class="size-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="M4 10l4-4 4 4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
        <button
          type="button"
          :class="ICON_BUTTON"
          :disabled="!neighbours.next"
          title="Next step (J)"
          aria-label="Next step"
          data-testid="decision-next"
          @click="neighbours.next && $emit('select', neighbours.next)"
        >
          <svg class="size-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
            <path d="M4 6l4 4 4-4" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
      </template>
      <button
        v-if="focused !== undefined"
        type="button"
        :class="ICON_BUTTON"
        :title="focused ? 'Show the steps again' : 'Hide the steps to read this one'"
        :aria-label="focused ? 'Show the steps' : 'Focus on this step'"
        :aria-pressed="focused"
        data-testid="decision-focus"
        @click="$emit('toggle-focus')"
      >
        <svg class="size-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path
            v-if="focused"
            d="M6.5 2.5v4h-4M9.5 13.5v-4h4M6.5 6.5 2.5 2.5M9.5 9.5l4 4"
            stroke="currentColor"
            stroke-width="1.3"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path
            v-else
            d="M2.5 6.5v-4h4M13.5 9.5v4h-4M2.5 2.5l4 4M13.5 13.5l-4-4"
            stroke="currentColor"
            stroke-width="1.3"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </button>
    </template>

    <template #facts>
      <StatusPill :tone="STATE_TONE[step.state]" :live="step.state === 'RUNNING'" data-testid="decision-state">
        {{ STATE_WORD[step.state] }}
      </StatusPill>
      <span data-testid="decision-position">Step {{ number }} of {{ total }}</span>
      <span v-if="stateMoment" class="text-text-subtle" data-testid="decision-moment">{{ stateMoment }}</span>
      <span v-if="step.passes.length" data-testid="decision-pass">Pass {{ passNumber }}</span>
      <span v-if="checksLine" :class="failures ? 'text-danger' : 'text-text-muted'" data-testid="decision-checks">
        <svg v-if="!failures" class="mr-1 inline size-3 -translate-y-px text-safe" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M2.2 6.4 4.8 9l5-5.6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        {{ checksLine }}
      </span>
      <span v-if="commitCount" data-testid="decision-commits">{{ commitCount }} {{ commitCount === 1 ? 'commit' : 'commits' }}</span>
      <button
        v-if="hasDeviation"
        type="button"
        class="focus-ring inline-flex items-center gap-1.5 rounded-key text-warn underline decoration-warn/40 underline-offset-3 hover:decoration-warn"
        :title="`Where the work left the plan on ${step.title}`"
        data-testid="step-deviation"
        @click="$emit('open-deviation', number)"
      >
        <span class="size-1.5 rounded-full bg-warn" aria-hidden="true" />
        Deviation
      </button>
    </template>

    <template #main><StepDetailBody :step="step" :task="task" /></template>
    <template #side><StepEvidence :step="step" :task="task" /></template>

    <template #dock>
      <p class="min-w-0 flex-1 text-small text-text-muted" data-testid="decision-note">{{ DOCK_NOTE[step.state] }}</p>
      <StepDecisionKeys
        :step="step"
        :task="task"
        :reopen-armed="reopenArmed"
        @accept="$emit('accept')"
        @send-back="$emit('send-back')"
        @reopen="$emit('reopen')"
        @mark-done="$emit('mark-done')"
      />
    </template>
  </DossierFrame>
</template>
