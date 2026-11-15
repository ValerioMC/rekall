<script setup lang="ts">
import { computed, toRef } from 'vue'
import StepDecisionKeys from '@/features/steps/components/StepDecisionKeys.vue'
import StepDetailBody from '@/features/steps/components/StepDetailBody.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import { useTaskDeviations } from '@/features/deviations/composables/useTaskDeviations'
import { useStepCheckRuns } from '@/features/check/composables/useStepCheckRuns'
import { useConsoleStore } from '@/stores/console.store'
import { relativeTime } from '@/common/format/relative-time'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'

/**
 * The selected step as a decision: a header that names the step, says what was found on it and
 * holds the keys a person decides with, and under it everything the step holds. Pinned, the
 * header stays while the body scrolls, so a long detail is read with the decision in reach. A
 * claim is accepted or sent back here; the other claims waiting on the task are listed to move
 * on to. Only a person marks a step done.
 */
const props = defineProps<{
  step: TaskStep
  task: Task
  number: number
  total: number
  reopenArmed: boolean
  /** The header stays and the body scrolls under it; unset, the panel grows with its text inside a page that scrolls. */
  pinned?: boolean
  /** Whether the step list is hidden so the panel has the whole tab. Unset where there is no list to hide. */
  focused?: boolean
  /** The steps either side of this one, to step through. Unset where there is no list to step through. */
  neighbours?: { readonly previous: TaskStep | null; readonly next: TaskStep | null }
  /** The other steps of the task a session has claimed, in checklist order. */
  upNext: readonly TaskStep[]
  numberOf: (step: TaskStep) => number
}>()

defineEmits<{
  accept: []
  'send-back': []
  reopen: []
  'mark-done': []
  select: [step: TaskStep]
  'toggle-focus': []
  'open-deviation': [number: number]
}>()

const store = useConsoleStore()
const { deviations } = useTaskDeviations(computed(() => props.task.id))
const { runs, failures } = useStepCheckRuns(toRef(props, 'step'))
const hasDeviation = computed(() => deviations.value.some((deviation) => deviation.stepNumber === props.number))
const commitCount = computed(() => store.stepCommitReferences(props.step.id).length)
const passNumber = computed(() => props.step.passes.length + 1)

const STATE_LINE: Readonly<Record<TaskStepState, string>> = {
  DRAFT: 'draft',
  OPEN: 'open',
  RUNNING: 'a session is working on it',
  CLAIMED: 'claimed',
  DONE: 'done'
}

const stateLine = computed(() => {
  const { step } = props
  const moment = step.state === 'CLAIMED' ? step.claimedAt : step.state === 'DONE' ? step.doneAt : null
  return moment ? `${STATE_LINE[step.state]} ${relativeTime(moment)}` : STATE_LINE[step.state]
})

const checksLine = computed(() => {
  if (!runs.value.length) return null
  return failures.value ? `${failures.value} of ${runs.value.length} checks failed` : `${runs.value.length} checks passed`
})

const headerClass = computed(() => (props.pinned ? 'shrink-0 border-b border-border px-8 pb-3.5 pt-4' : ''))
const bodyClass = computed(() => (props.pinned ? 'min-h-0 flex-1 overflow-y-auto px-8 pb-16 pt-6' : ''))
const columnClass = computed(() => (props.focused ? 'max-w-5xl' : 'max-w-4xl'))

const ICON_BUTTON =
  'focus-ring grid size-7 cursor-pointer place-items-center rounded-key text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text disabled:cursor-default disabled:opacity-40 disabled:hover:bg-transparent disabled:hover:text-text-subtle'
</script>

<template>
  <section
    class="flex flex-col gap-4"
    :class="pinned && 'h-full min-h-0 gap-0'"
    aria-label="Decision"
    data-testid="decision-panel"
    :data-step-state="step.state"
  >
    <header :class="headerClass">
      <div class="flex flex-wrap items-start gap-x-3 gap-y-2">
        <span class="mt-0.5 size-6 shrink-0" aria-hidden="true"><StepSeal :state="step.state" /></span>
        <h2
          class="min-w-[min(100%,16rem)] flex-1 text-heading font-medium leading-snug tracking-[-0.015em] text-text [text-wrap:balance]"
          data-testid="decision-title"
        >
          {{ step.title }}
        </h2>
        <div class="flex shrink-0 items-center gap-2">
          <StepDecisionKeys
            :step="step"
            :task="task"
            :reopen-armed="reopenArmed"
            @accept="$emit('accept')"
            @send-back="$emit('send-back')"
            @reopen="$emit('reopen')"
            @mark-done="$emit('mark-done')"
          />
          <div v-if="neighbours" class="flex items-center">
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
          </div>
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
        </div>
      </div>

      <p class="mt-1.5 flex flex-wrap items-center gap-x-3.5 gap-y-1 pl-9 text-small text-text-subtle" data-testid="decision-meta">
        <span data-testid="decision-state">Step {{ number }} of {{ total }} · {{ stateLine }}</span>
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
      </p>
    </header>

    <div :class="bodyClass">
      <div class="flex flex-col gap-4" :class="columnClass">
        <StepDetailBody :step="step" :task="task" />

        <div v-if="upNext.length" class="mt-2" data-testid="decision-up-next">
          <p class="eyebrow mb-1.5">Up next to review</p>
          <button
            v-for="next in upNext"
            :key="next.id"
            type="button"
            class="focus-ring grid min-h-9 w-full cursor-pointer grid-cols-[22px_minmax(0,1fr)] items-center gap-2 rounded-control px-1.5 text-left text-small text-text-muted transition-colors hover:bg-surface-hover hover:text-text"
            data-testid="decision-up-next-step"
            @click="$emit('select', next)"
          >
            <span class="font-mono text-caption tabular-nums text-text-subtle">{{ numberOf(next) }}</span>
            <span class="truncate">{{ next.title }}</span>
          </button>
        </div>
      </div>
    </div>
  </section>
</template>
