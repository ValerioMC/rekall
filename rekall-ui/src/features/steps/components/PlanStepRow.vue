<script setup lang="ts">
import { computed, ref } from 'vue'
import PlanStepMenu from '@/features/steps/components/PlanStepMenu.vue'
import StepPassBars from '@/features/steps/components/StepPassBars.vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import StepThread from '@/features/steps/components/StepThread.vue'
import { relativeTime } from '@/common/format/relative-time'
import type { StepPlaces } from '@/features/steps/lib/step-places'
import type { TaskStep } from '@/features/steps/model/task-step'

/**
 * One step of the plan as a row: its seal, its number, its title, and under it where the step
 * stands with a bar for each pass at it. Pressing the row selects the step for the decision panel.
 * The seal is the decision itself (accept a claim, tick an open step) and says so under the
 * pointer. An open step also has a handle to drag it to another place and a menu for the rare
 * moves; a step that has been started has neither, because there is nothing left to arrange.
 */
const props = defineProps<{
  step: TaskStep
  number: number
  selected: boolean
  isNext: boolean
  allDone: boolean
  /** Whether it is the last step of its session, where the thread ends. */
  endsSession: boolean
  /** Something is being dragged and this open step can take it next to itself. */
  droppable: boolean
  dropEdge: 'before' | 'after' | null
  dragging: boolean
  places: StepPlaces
}>()

const emit = defineEmits<{
  select: []
  'mark-forward': []
  'to-draft': []
  nudge: [by: number]
  'run-in': [sessionNumber: number]
  alone: [where: 'before-own-session' | 'end']
  remove: []
  'drag-start': []
  'drag-over': [after: boolean]
  drop: []
  'drag-end': []
}>()

const editable = computed(() => props.step.state === 'OPEN')
const handleHeld = ref(false)

const sealLabel = computed(() => {
  if (props.step.state === 'DONE') return `${props.step.title} is done`
  if (props.step.state === 'CLAIMED') return `Accept ${props.step.title}`
  return `Mark ${props.step.title} done`
})

const sealHint = computed(() => {
  if (props.step.state === 'CLAIMED') return `Accept step ${props.number}`
  return props.step.state === 'OPEN' ? `Mark step ${props.number} done` : ''
})

const standing = computed(() => {
  const { step } = props
  if (step.state === 'RUNNING') return 'Working now'
  if (step.state === 'CLAIMED') return step.claimedAt ? `Claimed ${relativeTime(step.claimedAt)}` : 'Claimed'
  if (step.state === 'DONE') return step.doneAt ? `Done ${relativeTime(step.doneAt)}` : 'Done'
  return 'Open'
})

function beginDrag(event: DragEvent): void {
  if (!handleHeld.value || !event.dataTransfer) {
    event.preventDefault()
    return
  }
  event.dataTransfer.effectAllowed = 'move'
  event.dataTransfer.setData('text/plain', props.step.id)
  emit('drag-start')
}

function hoverWhileDragging(event: DragEvent): void {
  if (!props.droppable) return
  event.preventDefault()
  const box = (event.currentTarget as HTMLElement).getBoundingClientRect()
  emit('drag-over', event.clientY > box.top + box.height / 2)
}

function dropHere(event: DragEvent): void {
  if (!props.droppable) return
  event.preventDefault()
  emit('drop')
}

function endDrag(): void {
  handleHeld.value = false
  emit('drag-end')
}
</script>

<template>
  <li
    class="group/step relative grid min-h-13 cursor-pointer grid-cols-[22px_22px_minmax(0,1fr)] items-start gap-x-2.5 rounded-control py-2 pl-2 transition-colors duration-(--motion-quick) focus-visible:outline-none"
    :class="[
      editable ? 'pr-11' : 'pr-2',
      selected ? 'selected-row' : 'hover:bg-surface-hover',
      step.state === 'RUNNING' && !selected && 'bg-session/10',
      dragging && 'opacity-40',
      dropEdge === 'before' && 'before:absolute before:inset-x-2 before:-top-px before:z-30 before:h-0.5 before:rounded-full before:bg-session-strong',
      dropEdge === 'after' && 'after:absolute after:inset-x-2 after:-bottom-px after:z-30 after:h-0.5 after:rounded-full after:bg-session-strong'
    ]"
    :draggable="handleHeld"
    :aria-current="selected ? 'true' : undefined"
    data-testid="plan-step"
    :data-step-id="step.id"
    :data-step-state="step.state"
    @click="emit('select')"
    @keydown.alt.up.prevent="editable && emit('nudge', -1)"
    @keydown.alt.down.prevent="editable && emit('nudge', 1)"
    @dragstart="beginDrag"
    @dragover="hoverWhileDragging"
    @drop="dropHere"
    @dragend="endDrag"
  >
    <StepThread placement="above" />
    <StepThread v-if="!endsSession" placement="below" />

    <button
      type="button"
      class="focus-ring hit-target group/seal relative z-10 size-[22px] rounded-full bg-canvas"
      :class="step.state === 'DONE' && 'cursor-default'"
      :aria-label="sealLabel"
      data-testid="plan-step-seal"
      data-seal-host
      @click.stop="step.state !== 'DONE' && emit('mark-forward')"
    >
      <StepSeal :state="step.state" :next="isNext" :complete="allDone" />
      <span
        v-if="sealHint"
        class="pointer-events-none absolute top-full left-0 z-(--z-overlay) mt-1.5 rounded-key border border-border-strong bg-surface-raised px-2.5 py-1 text-small font-medium whitespace-nowrap text-text opacity-0 shadow-(--shadow-lift) transition-opacity duration-(--motion-quick) group-hover/seal:opacity-100 group-hover/seal:delay-300 group-focus-visible/seal:opacity-100"
        role="tooltip"
        data-testid="plan-step-seal-hint"
      >
        {{ sealHint }}
      </span>
    </button>

    <span class="relative h-[22px]">
      <span
        class="absolute inset-0 pt-0.5 text-right font-mono text-caption tabular-nums text-text-subtle transition-opacity duration-(--motion-quick)"
        :class="editable && 'group-focus-within/step:opacity-0 group-hover/step:opacity-0'"
        data-testid="plan-step-number"
      >
        {{ number }}
      </span>
      <button
        v-if="editable"
        type="button"
        class="focus-ring absolute -right-1 top-px grid h-5 w-5 cursor-grab place-items-center rounded-key text-text-subtle opacity-0 transition-colors duration-(--motion-quick) group-focus-within/step:opacity-100 group-hover/step:opacity-100 hover:bg-surface-raised hover:text-text active:cursor-grabbing"
        :aria-label="`Drag step ${number} to another place, or hold Alt and press the arrow keys`"
        data-testid="step-handle"
        @pointerdown="handleHeld = true"
        @pointerup="handleHeld = false"
        @click.stop
      >
        <svg class="h-3.5 w-2.5" viewBox="0 0 10 14" fill="currentColor" aria-hidden="true">
          <circle cx="2.5" cy="2.5" r="1.2" />
          <circle cx="7.5" cy="2.5" r="1.2" />
          <circle cx="2.5" cy="7" r="1.2" />
          <circle cx="7.5" cy="7" r="1.2" />
          <circle cx="2.5" cy="11.5" r="1.2" />
          <circle cx="7.5" cy="11.5" r="1.2" />
        </svg>
      </button>
    </span>

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

    <span v-if="editable" class="absolute top-1.5 right-2" @click.stop>
      <PlanStepMenu
        :title="step.title"
        :number="number"
        :places="places"
        @to-draft="emit('to-draft')"
        @remove="emit('remove')"
        @nudge="(by) => emit('nudge', by)"
        @run-in="(sessionNumber) => emit('run-in', sessionNumber)"
        @alone="(where) => emit('alone', where)"
      />
    </span>
  </li>
</template>
