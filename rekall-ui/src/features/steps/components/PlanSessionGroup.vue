<script setup lang="ts">
import { computed, ref } from 'vue'
import StepSessionGap from '@/features/steps/components/StepSessionGap.vue'
import StepThread from '@/features/steps/components/StepThread.vue'
import { sessionNote, sessionStanding, type StepSession } from '@/features/steps/lib/step-sessions'
import { stepIsComplete, type TaskStep } from '@/features/steps/model/task-step'

/**
 * The steps that run in one Claude session, under a header that says where the session stands.
 * A thread through the seals ties them together; the seam between two of them cuts the session
 * there, and the header of a session that has not started can fold it into the one above. An
 * accepted session folds to its header while work remains elsewhere, and opens again when a step
 * in it is selected; once the whole plan is accepted nothing folds, so every step stays reachable.
 * A session none of whose steps has started has a grip on its header to move it whole; while any
 * session is held, all of them fold to their headers.
 */
const props = defineProps<{
  session: StepSession
  /** The selected step is one of this session's. */
  holdsSelection: boolean
  /** Some session of the plan is not accepted yet. */
  workRemains: boolean
  /** Every step is still open, so the session can be lifted whole. */
  movable: boolean
  /** A session is being dragged, so this one shows its header only. */
  foldedForDrag: boolean
}>()

const emit = defineEmits<{
  join: []
  split: [step: TaskStep]
  'drag-start': []
  'drag-end': []
  nudge: [by: number]
}>()

defineSlots<{ step(props: { step: TaskStep; endsSession: boolean }): unknown }>()

const standing = computed(() => sessionStanding(props.session.steps))
const folded = ref<boolean | null>(null)
const gripHeld = ref(false)
const open = computed(
  () =>
    !props.foldedForDrag &&
    (folded.value ?? (standing.value !== 'accepted' || !props.workRemains || props.holdsSelection))
)
const opener = computed(() => props.session.steps[0])
const canJoinAbove = computed(
  () => props.session.number > 1 && opener.value?.startsSession === true && !stepIsComplete(opener.value.state)
)

function toggle(): void {
  folded.value = !open.value
}

function beginDrag(event: DragEvent): void {
  if (!gripHeld.value || !event.dataTransfer) {
    event.preventDefault()
    return
  }
  event.dataTransfer.effectAllowed = 'move'
  event.dataTransfer.setData('text/plain', `session-${props.session.number}`)
  emit('drag-start')
}

function endDrag(): void {
  gripHeld.value = false
  emit('drag-end')
}

function endsSession(step: TaskStep): boolean {
  return props.session.steps[props.session.steps.length - 1]?.id === step.id
}
</script>

<template>
  <section
    class="group/session relative mt-4 first:mt-0"
    :data-standing="standing"
    :aria-label="`Session ${session.number}`"
    data-testid="plan-session"
  >
    <div
      class="relative grid min-h-9 grid-cols-[22px_minmax(0,1fr)_auto] items-center gap-x-2.5 rounded-control px-2"
      :draggable="gripHeld"
      @dragstart="beginDrag"
      @dragend="endDrag"
    >
      <StepThread v-if="open" placement="from-orb" />
      <component
        :is="standing === 'accepted' ? 'button' : 'div'"
        class="col-span-2 grid grid-cols-subgrid items-center text-left"
        :class="standing === 'accepted' && 'focus-ring -mx-2 cursor-pointer rounded-control px-2 hover:bg-surface-hover'"
        :type="standing === 'accepted' ? 'button' : undefined"
        :aria-expanded="standing === 'accepted' ? open : undefined"
        data-testid="session-header"
        @click="standing === 'accepted' && toggle()"
      >
        <span
          class="relative z-10 grid size-[22px] place-items-center rounded-full border-[1.5px] bg-canvas"
          :class="{
            'border-border-strong text-safe': standing === 'accepted',
            'border-session text-session': standing === 'working',
            'border-dashed border-session bg-session/15 text-session-strong': standing === 'waiting',
            'border-border-field text-text-subtle': standing === 'idle'
          }"
          aria-hidden="true"
        >
          <svg v-if="standing === 'accepted'" class="size-3" viewBox="0 0 12 12" fill="none">
            <path d="M2.5 6.3l2.4 2.4 4.6-5" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
          <span v-else class="size-[7px] rounded-full bg-current" :class="standing === 'working' && 'animate-pulse motion-reduce:animate-none'" />
        </span>
        <span class="flex min-w-0 items-baseline gap-2.5">
          <b class="text-body font-semibold whitespace-nowrap text-text">Session {{ session.number }}</b>
          <span
            class="truncate text-caption"
            :class="standing === 'working' || standing === 'waiting' ? 'text-session-strong' : 'text-text-subtle'"
            data-testid="session-note"
          >
            {{ sessionNote(session.steps) }}
          </span>
          <svg
            v-if="standing === 'accepted'"
            class="size-2.5 shrink-0 self-center text-text-subtle transition-transform duration-(--motion-quick)"
            :class="open && 'rotate-90'"
            viewBox="0 0 24 24"
            fill="none"
            aria-hidden="true"
          >
            <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </span>
      </component>
      <span class="flex items-center gap-1.5">
        <span v-if="foldedForDrag && !movable" class="inline-flex items-center gap-1.5 text-caption text-text-subtle" data-testid="session-started">
          <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <rect x="2.5" y="5.5" width="7" height="5" rx="1" stroke="currentColor" stroke-width="1.2" />
            <path d="M4 5.5V4a2 2 0 014 0v1.5" stroke="currentColor" stroke-width="1.2" />
          </svg>
          Started
        </span>
        <button
          v-if="canJoinAbove && !foldedForDrag"
          type="button"
          class="key-quiet focus-ring inline-flex h-6 cursor-pointer items-center gap-1.5 rounded-key px-2.5 text-small font-medium opacity-0 transition-opacity duration-(--motion-quick) group-focus-within/session:opacity-100 group-hover/session:opacity-100"
          :aria-label="`Run session ${session.number} in session ${session.number - 1}`"
          data-testid="session-join"
          @click="emit('join')"
        >
          Join session above
        </button>
        <button
          v-if="movable"
          type="button"
          class="focus-ring grid size-6 cursor-grab place-items-center rounded-key text-text-subtle opacity-0 transition-colors duration-(--motion-quick) group-focus-within/session:opacity-100 group-hover/session:opacity-100 hover:bg-surface-raised hover:text-text active:cursor-grabbing"
          :aria-label="`Drag session ${session.number} to another place, or hold Alt and press the arrow keys`"
          data-testid="session-handle"
          @pointerdown="gripHeld = true"
          @pointerup="gripHeld = false"
          @keydown.alt.up.prevent="emit('nudge', -1)"
          @keydown.alt.down.prevent="emit('nudge', 1)"
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
    </div>

    <ol v-if="open" class="flex flex-col" data-testid="session-steps">
      <template v-for="(step, index) in session.steps" :key="step.id">
        <StepSessionGap v-if="index > 0 && !stepIsComplete(step.state)" :title="step.title" @split="emit('split', step)">
          <template #thread><StepThread placement="through" /></template>
        </StepSessionGap>
        <li v-else-if="index > 0" class="relative h-1.5" role="presentation" aria-hidden="true">
          <StepThread placement="through" />
        </li>
        <slot name="step" :step="step" :ends-session="endsSession(step)" />
      </template>
    </ol>
  </section>
</template>
