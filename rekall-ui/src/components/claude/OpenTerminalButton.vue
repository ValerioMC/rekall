<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/stores/terminal.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { preferredEffort, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import { useToastStore } from '@/stores/toast.store'
import { sessionAtWork } from '@/model/session-at-work'
import StepSeal from '@/components/console/StepSeal.vue'
import type { TaskId, TaskStepId } from '@/model/branded'

/**
 * Starts the task's session without leaving the pane it was pressed on; the sessions dock is the
 * way back to it. One glyph that changes: `idle` a play mark, `launching` the mark runs out
 * through the edge, `waiting` an open ring turning while the server answers, `done` the
 * step's green seal settling in, `working` a comet orbiting until the session claims the work or ends. Only
 * `idle` takes a press, and the button keeps its width throughout.
 */
const props = withDefaults(
  defineProps<{
    taskId: TaskId
    folder: string | null
    stepId?: TaskStepId | null
    missingHint?: string
  }>(),
  { stepId: null, missingHint: "Set this project's folder on its page to open a terminal from it" }
)

type LaunchPhase = 'idle' | 'launching' | 'waiting' | 'done'
type ButtonPhase = LaunchPhase | 'working'

/** Long enough for the play mark to clear the button before the ring takes its place. */
const TRAVEL_MS = 380
const DONE_HOLD_MS = 1200
/** How long `working` holds after a launch while the server's RUNNING mark is still on its way. */
const HANDOFF_MS = 8000

const console_ = useConsoleStore()
const terminals = useTerminalStore()
const toast = useToastStore()
const { run } = useAsyncAction()
const { tasks, steps } = storeToRefs(console_)
const { terminals: openTerminals } = storeToRefs(terminals)

const launchPhase = ref<LaunchPhase>('idle')
const handingOff = ref(false)
const returning = ref(false)
const pendingTimers = new Set<number>()

const ready = computed(() => Boolean(props.folder))
const atWork = computed(() =>
  sessionAtWork(
    tasks.value.find((task) => task.id === props.taskId) ?? null,
    steps.value,
    openTerminals.value,
    props.stepId
  )
)
const phase = computed<ButtonPhase>(() => {
  if (launchPhase.value !== 'idle') return launchPhase.value
  return atWork.value || handingOff.value ? 'working' : 'idle'
})
const busy = computed(() => phase.value !== 'idle')

watch(atWork, (working) => {
  if (working) handingOff.value = false
})

watch(phase, (next, previous) => {
  returning.value = next === 'idle' && previous !== 'idle'
})

function pause(milliseconds: number): Promise<void> {
  return new Promise((resolve) => {
    const timer = window.setTimeout(() => {
      pendingTimers.delete(timer)
      resolve()
    }, milliseconds)
    pendingTimers.add(timer)
  })
}

async function launch(): Promise<void> {
  if (busy.value) return
  if (!ready.value) {
    toast.notifyError(new Error(props.missingHint))
    return
  }
  launchPhase.value = 'launching'
  const opening = run(() =>
    terminals.openForTask(props.taskId, {
      stepId: props.stepId,
      skipPermissions: skipsPermissions(),
      model: preferredModel(),
      effort: preferredEffort()
    })
  )
  await pause(TRAVEL_MS)
  // Painted only if the server is still answering: a settled `opening` moves on in the same tick.
  launchPhase.value = 'waiting'
  const opened = await opening
  if (opened === null) {
    launchPhase.value = 'idle'
    return
  }
  launchPhase.value = 'done'
  await pause(DONE_HOLD_MS)
  handingOff.value = !atWork.value
  launchPhase.value = 'idle'
  if (handingOff.value) {
    await pause(HANDOFF_MS)
    handingOff.value = false
  }
}

onBeforeUnmount(() => {
  pendingTimers.forEach((timer) => window.clearTimeout(timer))
  pendingTimers.clear()
})
</script>

<template>
  <button
    type="button"
    class="focus-ring relative inline-flex h-7 shrink-0 items-center gap-1.5 overflow-hidden rounded-[3px] px-2.5 text-xs font-medium"
    :class="[
      phase === 'done'
        ? 'key-quiet'
        : ready
          ? phase === 'working'
            ? 'key-lit'
            : 'key-gold font-semibold'
          : 'bg-transparent text-text-subtle transition-colors hover:bg-surface-raised hover:text-text-muted',
      busy && 'cursor-default'
    ]"
    :aria-busy="busy"
    :aria-disabled="busy"
    :title="phase === 'working' ? 'Claude is working on this. It returns once the work is claimed or the session ends.' : undefined"
    :data-phase="phase"
    data-testid="open-terminal"
    @click="launch"
  >
    <span class="grid size-3.5 [&>*]:[grid-area:1/1]">
      <svg
        class="size-3.5"
        :class="{
          'run-play-return': phase === 'idle' && returning,
          'run-play-away': phase !== 'idle' && phase !== 'working',
          invisible: phase === 'working'
        }"
        viewBox="0 0 12 12"
        aria-hidden="true"
      >
        <path
          d="M3.4 1.9c0-.6.66-.97 1.17-.65l5.9 3.75c.47.3.47.99 0 1.29l-5.9 3.76c-.51.32-1.17-.05-1.17-.65Z"
          fill="currentColor"
        />
      </svg>
      <svg v-if="phase === 'working'" class="run-orbit size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
        <circle class="run-orbit-track" cx="7" cy="7" r="5.6" />
        <g class="run-orbit-comet">
          <circle class="run-orbit-tail" cx="7" cy="7" r="5.6" pathLength="100" />
          <circle class="run-orbit-head" cx="7" cy="7" r="5.6" pathLength="100" />
        </g>
        <circle class="run-orbit-core" cx="7" cy="7" r="1.7" />
      </svg>
    </span>
    <span class="grid [&>*]:[grid-area:1/1]">
      <span
        :class="{
          'run-label-return': phase === 'idle' && returning,
          'run-label-away': phase !== 'idle' && phase !== 'working',
          invisible: phase === 'working'
        }"
        >Run here</span
      >
      <span :class="phase === 'working' ? 'run-label-in' : 'invisible'" aria-hidden="true">Working</span>
    </span>

    <span
      v-if="phase === 'waiting' || phase === 'done'"
      class="absolute inset-0 grid place-items-center"
      aria-hidden="true"
    >
      <StepSeal v-if="phase === 'done'" class="run-seal size-[18px]" state="DONE" complete />
      <svg v-else class="size-4" viewBox="0 0 16 16" fill="none">
        <circle
          class="run-ring run-ring-open"
          cx="8"
          cy="8"
          r="6.4"
          pathLength="100"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linecap="round"
        />
      </svg>
    </span>

    <span class="sr-only" aria-live="polite">{{
      phase === 'done' ? 'Session started' : phase === 'working' ? 'Claude is working' : ''
    }}</span>
  </button>
</template>

<style scoped>
/* Accelerating, like something leaving: the mark eases out of rest and is gone past the edge. */
.run-play-away {
  animation: run-play-away 380ms cubic-bezier(0.55, 0, 0.75, 0.2) forwards;
}

.run-label-away {
  animation: run-label-away 180ms ease-out forwards;
}

.run-play-return,
.run-label-return {
  animation: fade-in 220ms ease-out;
}

/* The ring rotates about its own centre, not the SVG origin. */
.run-ring {
  transform-box: fill-box;
  transform-origin: center;
}

.run-ring-open {
  stroke-dasharray: 28 72;
  opacity: 0.8;
  animation: dial-sweep 800ms linear infinite;
}

/* The step's own seal, set into the key: it settles in rather than appearing. */
.run-seal {
  animation: run-seal-in 260ms cubic-bezier(0.3, 0.7, 0.2, 1) both;
}

/* The seal's RUNNING motion at button size: a comet orbits a faint track and the core breathes. */
.run-orbit {
  animation: fade-in 260ms ease-out both;
}

.run-orbit-track {
  stroke: color-mix(in srgb, var(--color-accent) 26%, transparent);
  stroke-width: 1.2;
}

.run-orbit-comet {
  transform-box: view-box;
  transform-origin: 7px 7px;
  animation: run-orbit 1.7s linear infinite;
}

.run-orbit-tail,
.run-orbit-head {
  stroke-linecap: round;
  transform-box: view-box;
  transform-origin: 7px 7px;
  transform: rotate(-90deg);
}

.run-orbit-tail {
  stroke: color-mix(in srgb, var(--color-accent) 70%, transparent);
  stroke-width: 1.4;
  stroke-dasharray: 25 75;
}

/* The head rides the leading end of the tail. */
.run-orbit-head {
  stroke: var(--color-accent-strong);
  stroke-width: 2;
  stroke-dasharray: 5 95;
  stroke-dashoffset: -20;
}

.run-orbit-core {
  fill: var(--color-accent);
  transform-box: fill-box;
  transform-origin: center;
  animation: run-breathe 1.7s ease-in-out infinite;
}

.run-label-in {
  animation: fade-in 260ms ease-out both;
}

@keyframes run-play-away {
  0% {
    transform: translateX(0);
    opacity: 1;
  }
  70% {
    opacity: 1;
  }
  100% {
    transform: translateX(72px);
    opacity: 0;
  }
}

@keyframes run-label-away {
  to {
    opacity: 0;
    transform: translateX(6px);
  }
}

@keyframes run-orbit {
  to {
    transform: rotate(360deg);
  }
}

@keyframes run-breathe {
  50% {
    transform: scale(0.7);
    opacity: 0.6;
  }
}

@keyframes run-seal-in {
  from {
    opacity: 0;
    transform: scale(0.6);
  }
}
</style>
