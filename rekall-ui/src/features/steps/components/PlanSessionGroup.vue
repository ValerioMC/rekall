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
 */
const props = defineProps<{
  session: StepSession
  /** The selected step is one of this session's. */
  holdsSelection: boolean
  /** Some session of the plan is not accepted yet. */
  workRemains: boolean
}>()

const emit = defineEmits<{ join: []; split: [step: TaskStep] }>()

defineSlots<{ step(props: { step: TaskStep; endsSession: boolean }): unknown }>()

const standing = computed(() => sessionStanding(props.session.steps))
const folded = ref<boolean | null>(null)
const open = computed(() => folded.value ?? (standing.value !== 'accepted' || !props.workRemains || props.holdsSelection))
const opener = computed(() => props.session.steps[0])
const canJoinAbove = computed(
  () => props.session.number > 1 && opener.value?.startsSession === true && !stepIsComplete(opener.value.state)
)

function toggle(): void {
  folded.value = !open.value
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
    <div class="relative grid min-h-9 grid-cols-[22px_minmax(0,1fr)_auto] items-center gap-x-2.5 rounded-control px-2">
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
      <button
        v-if="canJoinAbove"
        type="button"
        class="key-quiet focus-ring inline-flex h-6 cursor-pointer items-center gap-1.5 rounded-key px-2.5 text-small font-medium opacity-0 transition-opacity duration-(--motion-quick) group-focus-within/session:opacity-100 group-hover/session:opacity-100"
        :aria-label="`Run session ${session.number} in session ${session.number - 1}`"
        data-testid="session-join"
        @click="emit('join')"
      >
        Join session above
      </button>
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
