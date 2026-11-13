<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import OpenTerminalButton from '@/features/claude/components/OpenTerminalButton.vue'
import StepDetailBody from '@/features/steps/components/StepDetailBody.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import { useTaskDeviations } from '@/features/deviations/composables/useTaskDeviations'
import { relativeTime } from '@/common/format/relative-time'
import type { Task } from '@/features/catalog/model/catalog'
import type { TaskStep, TaskStepState } from '@/features/steps/model/task-step'

/**
 * The selected step as a decision: what state it is in, what a person can do about it, and under
 * that everything the step holds. A claim is accepted or sent back here; the other claims waiting
 * on the task are listed to move on to. Only a person marks a step done.
 */
const props = defineProps<{
  step: TaskStep
  task: Task
  number: number
  total: number
  reopenArmed: boolean
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
  'open-deviation': [number: number]
}>()

const { deviations } = useTaskDeviations(computed(() => props.task.id))
const hasDeviation = computed(() => deviations.value.some((deviation) => deviation.stepNumber === props.number))

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
</script>

<template>
  <section class="flex flex-col gap-4" aria-label="Decision" data-testid="decision-panel" :data-step-state="step.state">
    <header>
      <p class="flex items-center gap-2 text-caption text-text-subtle">
        <span class="size-5.5" aria-hidden="true"><StepSeal :state="step.state" /></span>
        <span data-testid="decision-kicker">Step {{ number }} of {{ total }} · {{ stateLine }}</span>
        <button
          v-if="hasDeviation"
          type="button"
          class="focus-ring ml-auto rounded-key text-accent underline decoration-accent/40 underline-offset-2 hover:decoration-accent"
          :title="`Where the work left the plan on ${step.title}`"
          data-testid="step-deviation"
          @click="$emit('open-deviation', number)"
        >
          Deviation
        </button>
      </p>
      <h2 class="mt-2 text-heading font-semibold tracking-[-0.015em] text-text" data-testid="decision-title">
        {{ step.title }}
      </h2>
    </header>

    <div v-if="step.state === 'OPEN' || step.state === 'RUNNING'" class="flex flex-wrap items-center gap-2" data-testid="decision-actions">
      <AppButton size="sm" data-testid="decision-mark-done" @click="$emit('mark-done')">Mark done myself</AppButton>
      <OpenTerminalButton
        :task-id="task.id"
        :anchor="task.anchor"
        :step-id="step.id"
        :folder="task.projectRepoFolder"
      />
    </div>

    <StepDetailBody
      :step="step"
      :task="task"
      :reopen-armed="reopenArmed"
      @accept="$emit('accept')"
      @send-back="$emit('send-back')"
      @reopen="$emit('reopen')"
    />

    <div v-if="upNext.length" data-testid="decision-up-next">
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
  </section>
</template>
