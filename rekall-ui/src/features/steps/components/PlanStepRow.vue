<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import StepPassBars from '@/features/steps/components/StepPassBars.vue'
import StepRowControls from '@/features/steps/components/StepRowControls.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import { relativeTime } from '@/common/format/relative-time'
import { stepIsComplete, type TaskStep } from '@/features/steps/model/task-step'

/**
 * One step of the plan as a row: its seal, its number, its title, and under it where the step
 * stands with a bar for each pass at it. Pressing the row selects the step for the decision panel;
 * the seal and the hover key are the decision itself (accept a claim, tick an open step), and the
 * hover controls arrange the step.
 */
const props = defineProps<{
  step: TaskStep
  number: number
  selected: boolean
  isNext: boolean
  allDone: boolean
  isLast: boolean
  /** False for the first step of the checklist, which always opens the first session. */
  canChooseSession: boolean
}>()

const emit = defineEmits<{
  select: []
  'mark-forward': []
  'to-draft': []
  move: [by: number]
  remove: []
  session: [startsSession: boolean]
}>()

const sealLabel = computed(() => {
  if (props.step.state === 'DONE') return `${props.step.title} is done`
  if (props.step.state === 'CLAIMED') return `Accept ${props.step.title}`
  return `Mark ${props.step.title} done`
})

const standing = computed(() => {
  const { step } = props
  if (step.state === 'RUNNING') return 'Working now'
  if (step.state === 'CLAIMED') return step.claimedAt ? `Claimed ${relativeTime(step.claimedAt)}` : 'Claimed'
  if (step.state === 'DONE') return step.doneAt ? `Done ${relativeTime(step.doneAt)}` : 'Done'
  return 'Open'
})

const rowKey = computed(() => {
  if (props.step.state === 'CLAIMED') return 'Accept'
  return props.step.state === 'OPEN' ? 'Done' : ''
})
</script>

<template>
  <li
    class="group/step relative grid min-h-13 cursor-pointer grid-cols-[22px_22px_minmax(0,1fr)] items-start gap-x-2.5 rounded-control px-2 py-2 transition-colors duration-(--motion-quick) focus-visible:outline-none"
    :class="[
      selected ? 'selected-row' : 'hover:bg-surface-hover',
      step.state === 'RUNNING' && !selected && 'bg-session/10'
    ]"
    data-testid="plan-step"
    :data-step-id="step.id"
    :data-step-state="step.state"
    @click="emit('select')"
  >
    <button
      type="button"
      class="focus-ring hit-target relative size-[22px] rounded-full"
      :class="step.state === 'DONE' && 'cursor-default'"
      :aria-label="sealLabel"
      data-testid="plan-step-seal"
      data-seal-host
      @click.stop="step.state !== 'DONE' && emit('mark-forward')"
    >
      <StepSeal :state="step.state" :next="isNext" :complete="allDone" />
    </button>

    <span class="pt-0.5 font-mono text-caption tabular-nums text-text-subtle" data-testid="plan-step-number">{{ number }}</span>

    <span class="flex min-w-0 flex-col gap-0.5">
      <button
        type="button"
        class="focus-ring line-clamp-2 rounded-key text-left text-body"
        :class="step.state === 'DONE' ? 'text-text-subtle line-through decoration-text-subtle/50' : 'text-text'"
        :aria-current="selected ? 'true' : undefined"
        data-testid="plan-step-title"
        @click.stop="emit('select')"
      >
        {{ step.title }}
      </button>
      <span class="flex items-center gap-2.5 text-caption">
        <span
          class="min-w-0 flex-1 truncate"
          :class="step.state === 'CLAIMED' ? 'font-medium text-session-strong' : step.state === 'RUNNING' ? 'text-session-strong' : 'text-text-subtle'"
          data-testid="plan-step-standing"
        >
          {{ standing }}
        </span>
        <StepPassBars :step="step" />
      </span>
    </span>

    <span
      class="absolute inset-y-1 right-1 flex items-center gap-0.5 rounded-control pl-2 pr-1 opacity-0 transition-opacity duration-(--motion-quick) group-focus-within/step:opacity-100 group-hover/step:opacity-100 pointer-events-none group-focus-within/step:pointer-events-auto group-hover/step:pointer-events-auto"
      :class="selected ? 'bg-surface-raised' : 'bg-surface-hover'"
      @click.stop
    >
      <button
        v-if="canChooseSession && !stepIsComplete(step.state)"
        type="button"
        class="focus-ring grid h-6 place-items-center rounded-key px-1.5 text-micro font-medium transition-colors hover:bg-surface-hover hover:text-text"
        :aria-label="step.startsSession ? `Run ${step.title} in the session above` : `Open a new session on ${step.title}`"
        data-testid="step-session"
        @click="emit('session', !step.startsSession)"
      >
        {{ step.startsSession ? 'Join session above' : 'New session' }}
      </button>
      <button
        v-if="step.state === 'OPEN'"
        type="button"
        class="focus-ring grid h-6 place-items-center rounded-key px-1.5 text-micro font-medium transition-colors hover:bg-surface-hover hover:text-text"
        :aria-label="`Return ${step.title} to draft`"
        data-testid="step-to-draft"
        @click="emit('to-draft')"
      >
        To draft
      </button>
      <StepRowControls
        :title="step.title"
        :can-move-up="step.position > 0"
        :can-move-down="!isLast"
        testid-prefix="step"
        @up="emit('move', -1)"
        @down="emit('move', 1)"
        @remove="emit('remove')"
      />
      <AppButton
        v-if="rowKey"
        size="sm"
        :variant="step.state === 'CLAIMED' ? 'primary' : 'secondary'"
        data-testid="plan-step-key"
        @click="emit('mark-forward')"
      >
        {{ rowKey }}
      </AppButton>
    </span>
  </li>
</template>
