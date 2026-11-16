<script setup lang="ts">
import { computed, nextTick, ref, toRef, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import StatusPill, { type StatusPillTone } from '@/components/ui/StatusPill.vue'
import LogCommitButton from '@/features/commits/components/LogCommitButton.vue'
import DossierFrame from '@/features/steps/components/DossierFrame.vue'
import StepDecisionKeys from '@/features/steps/components/StepDecisionKeys.vue'
import StepDetailBody from '@/features/steps/components/StepDetailBody.vue'
import StepEvidence from '@/features/steps/components/StepEvidence.vue'
import StepHistory from '@/features/steps/components/StepHistory.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import StepSetup from '@/features/steps/components/StepSetup.vue'
import { useTaskDeviations } from '@/features/deviations/composables/useTaskDeviations'
import { useStepCheckRuns } from '@/features/check/composables/useStepCheckRuns'
import { checksLine as checksLineOf } from '@/features/check/lib/check-results'
import { useStepEditor } from '@/features/steps/composables/useStepEditor'
import { useConsoleStore } from '@/stores/console.store'
import { relativeTime } from '@/common/format/relative-time'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'

/**
 * The selected step as a decision: a header that names the step (the title is renamed in place),
 * switches the detail between Write and Read and says what was found on it; the detail taking the
 * width of the pane; the evidence in a drawer the facts open, or in a rail when the list is hidden
 * on a wide pane; and a dock with the keys a person decides with, only while a decision is still
 * to be made. Only a person marks a step done.
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
  /** A CSS `max-width` that holds the panel to a centred column; unset it fills the pane. */
  maxWidth?: string
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
const editor = useStepEditor()
const { deviations } = useTaskDeviations(computed(() => props.task.id))
const { runs, failures } = useStepCheckRuns(toRef(props, 'step'))
const hasDeviation = computed(() => deviations.value.some((deviation) => deviation.stepNumber === props.number))
const commitCount = computed(() => store.stepCommitReferences(props.step.id).length)
const hasEvidence = computed(() => runs.value.length > 0 || commitCount.value > 0)
const passNumber = computed(() => props.step.passes.length + 1)
const awaitsDecision = computed(() => props.step.state !== 'DONE' && props.step.state !== 'DRAFT')
const canLogCommit = computed(() => props.step.state === 'RUNNING' || props.step.state === 'CLAIMED')

const evidenceOpen = ref(false)
const renaming = ref(false)
const titleField = ref<HTMLInputElement | null>(null)

// A claim is opened on its evidence, since the decision is read from it; so is a failed check.
watch(
  () => props.step.id,
  () => {
    evidenceOpen.value = props.step.state === 'CLAIMED' || failures.value > 0
    renaming.value = false
  },
  { immediate: true }
)

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

const stateMoment = computed(() => {
  const { step } = props
  const moment = step.state === 'CLAIMED' ? step.claimedAt : step.state === 'DONE' ? step.doneAt : null
  return moment ? relativeTime(moment) : null
})

const checksLine = computed(() => checksLineOf(runs.value))

async function startRenaming(): Promise<void> {
  renaming.value = true
  await nextTick()
  titleField.value?.focus()
  titleField.value?.select()
}

function stopRenaming(): void {
  if (!editor.title.value.trim()) editor.title.value = props.step.title
  renaming.value = false
}

const ICON_BUTTON =
  'focus-ring grid size-7 cursor-pointer place-items-center rounded-key text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text disabled:cursor-default disabled:opacity-40 disabled:hover:bg-transparent disabled:hover:text-text-subtle'
const FACT_BUTTON =
  'focus-ring -mx-2 inline-flex h-6 cursor-pointer items-center gap-1.5 rounded-control px-2 transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text aria-expanded:bg-surface-raised aria-expanded:text-text'
</script>

<template>
  <DossierFrame :pinned="pinned" :focused="focused === true" :max-width="maxWidth" aria-label="Decision" data-testid="decision-panel" :data-step-state="step.state">
    <template #lead><StepSeal :state="step.state" /></template>

    <template #title>
      <input
        v-if="renaming"
        ref="titleField"
        v-model="editor.title.value"
        class="-my-0.5 w-full rounded-control border border-accent bg-canvas px-2 py-0.5 text-heading font-medium tracking-[-0.015em] text-text outline-none ring-1 ring-accent"
        :aria-label="`Title of ${step.title}`"
        data-testid="step-title-field"
        @input="editor.scheduleSave()"
        @blur="stopRenaming"
        @keydown.enter.prevent="stopRenaming"
        @keydown.esc.stop="stopRenaming"
      />
      <button
        v-else
        type="button"
        class="focus-ring group/title -mx-1 rounded-key px-1 text-left transition-colors duration-(--motion-quick) hover:bg-surface-raised"
        title="Rename the step"
        data-testid="decision-title"
        @click="startRenaming"
      >
        {{ step.title }}
        <svg class="ml-2 inline size-3.5 -translate-y-px text-text-subtle opacity-0 group-hover/title:opacity-100 group-focus-visible/title:opacity-100" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <path d="M10.5 3.5l2 2L5 13H3v-2z" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </button>
    </template>

    <template #controls>
      <slot name="controls" />
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

    <template #facts="{ railed }">
      <StatusPill :tone="STATE_TONE[step.state]" :live="step.state === 'RUNNING'" data-testid="decision-state">
        {{ STATE_WORD[step.state] }}
      </StatusPill>
      <span data-testid="decision-position">Step {{ number }} of {{ total }}</span>
      <span v-if="stateMoment" class="text-text-subtle" data-testid="decision-moment">{{ stateMoment }}</span>
      <span v-if="step.passes.length" data-testid="decision-pass">Pass {{ passNumber }}</span>
      <template v-if="checksLine">
        <span v-if="railed" :class="failures ? 'text-danger' : 'text-text-muted'" data-testid="decision-checks">{{ checksLine }}</span>
        <button
          v-else
          type="button"
          :class="[FACT_BUTTON, failures ? 'text-danger' : 'text-text-muted']"
          :aria-expanded="evidenceOpen"
          data-testid="decision-checks"
          @click="evidenceOpen = !evidenceOpen"
        >
          <svg v-if="!failures" class="size-3 text-safe" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M2.2 6.4 4.8 9l5-5.6" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
          {{ checksLine }}
          <svg class="size-2.5 text-text-subtle transition-transform" :class="evidenceOpen && 'rotate-90'" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
      </template>
      <template v-if="commitCount">
        <span v-if="railed" data-testid="decision-commits">{{ commitCount }} {{ commitCount === 1 ? 'commit' : 'commits' }}</span>
        <button
          v-else
          type="button"
          :class="[FACT_BUTTON, 'text-text-muted']"
          :aria-expanded="evidenceOpen"
          data-testid="decision-commits"
          @click="evidenceOpen = !evidenceOpen"
        >
          {{ commitCount }} {{ commitCount === 1 ? 'commit' : 'commits' }}
          <svg class="size-2.5 text-text-subtle transition-transform" :class="evidenceOpen && 'rotate-90'" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
      </template>
      <LogCommitButton v-if="canLogCommit" variant="bar" :task-id="task.id" :step-id="step.id" :folder="task.projectRepoFolder" />
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
      <span class="ml-auto flex items-center gap-1.5">
        <AppModeToggle v-model="editor.mode.value" class="mr-1" testid-prefix="step-detail" />
        <AppButton
          v-if="step.state === 'DONE'"
          variant="quiet"
          size="sm"
          :class="reopenArmed && 'text-danger'"
          title="Drops it back to open and clears the review it passed"
          data-testid="step-reopen"
          @click="$emit('reopen')"
        >
          {{ reopenArmed ? 'Confirm reopen' : 'Reopen' }}
        </AppButton>
        <AppButton v-if="focused !== undefined" variant="quiet" size="sm" title="Back to the overview (Esc)" data-testid="decision-overview" @click="$emit('overview')">
          Overview
          <kbd class="font-mono text-micro text-text-subtle">Esc</kbd>
        </AppButton>
      </span>
    </template>

    <template #drawer="{ railed }">
      <div v-if="evidenceOpen && !railed && hasEvidence" class="border-b border-border bg-canvas" data-testid="step-evidence">
        <div class="grid w-full grid-cols-[repeat(auto-fit,minmax(22rem,1fr))] items-start gap-x-10 gap-y-4 px-7 pb-2 pt-4">
          <StepEvidence :step="step" />
        </div>
      </div>
    </template>

    <template #main="{ railed }">
      <StepDetailBody :step="step" :task="task" />
      <StepSetup v-if="!railed" :step="step" :task="task" />
    </template>

    <template #rail>
      <StepHistory :step="step" />
      <StepEvidence :step="step" />
      <StepSetup :step="step" :task="task" stacked />
    </template>

    <template v-if="awaitsDecision" #dock>
      <StepDecisionKeys
        :step="step"
        :task="task"
        @accept="$emit('accept')"
        @send-back="$emit('send-back')"
        @mark-done="$emit('mark-done')"
      />
    </template>
  </DossierFrame>
</template>
