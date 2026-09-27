<script setup lang="ts">
import { computed, onBeforeUnmount, ref } from 'vue'
import { useTerminalStore } from '@/stores/terminal.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { preferredEffort, preferredModel, skipsPermissions } from '@/common/config/claude-launch'
import { useToastStore } from '@/stores/toast.store'
import type { TaskId, TaskStepId } from '@/model/branded'

/**
 * Starts the task's session without leaving the pane it was pressed on; the sessions dock is the
 * way back to it. The launch is one glyph that changes: `idle` a play mark, `launching` the mark
 * runs out through the button's edge, `waiting` an open ring turning while the server answers,
 * `done` the ring closing on a check. The button keeps its width throughout.
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

/** Long enough for the play mark to clear the button before the ring takes its place. */
const TRAVEL_MS = 380
const DONE_HOLD_MS = 2200

const terminals = useTerminalStore()
const toast = useToastStore()
const { run } = useAsyncAction()

const phase = ref<LaunchPhase>('idle')
const hasLaunched = ref(false)
const pendingTimers = new Set<number>()

const ready = computed(() => Boolean(props.folder))
const busy = computed(() => phase.value !== 'idle')

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
  phase.value = 'launching'
  hasLaunched.value = true
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
  phase.value = 'waiting'
  const opened = await opening
  if (opened === null) {
    phase.value = 'idle'
    return
  }
  phase.value = 'done'
  await pause(DONE_HOLD_MS)
  phase.value = 'idle'
}

onBeforeUnmount(() => {
  pendingTimers.forEach((timer) => window.clearTimeout(timer))
  pendingTimers.clear()
})
</script>

<template>
  <button
    type="button"
    class="focus-ring relative inline-flex h-7 shrink-0 items-center gap-1.5 overflow-hidden rounded-[var(--radius-control)] border px-2.5 text-xs font-medium transition-colors duration-200 active:translate-y-px"
    :class="[
      phase === 'done'
        ? 'run-done border-transparent text-safe'
        : ready
          ? 'border-accent bg-accent-soft text-accent'
          : 'border-transparent bg-transparent text-text-subtle hover:bg-surface-raised hover:text-text-muted',
      ready && phase === 'idle' && 'hover:bg-accent hover:text-accent-ink',
      busy && 'cursor-default'
    ]"
    :aria-busy="phase === 'launching' || phase === 'waiting'"
    :aria-disabled="busy"
    :data-phase="phase"
    data-testid="open-terminal"
    @click="launch"
  >
    <svg
      class="size-3.5"
      :class="phase === 'idle' ? hasLaunched && 'run-play-return' : 'run-play-away'"
      viewBox="0 0 12 12"
      aria-hidden="true"
    >
      <path
        d="M3.4 1.9c0-.6.66-.97 1.17-.65l5.9 3.75c.47.3.47.99 0 1.29l-5.9 3.76c-.51.32-1.17-.05-1.17-.65Z"
        fill="currentColor"
      />
    </svg>
    <span :class="phase === 'idle' ? hasLaunched && 'run-label-return' : 'run-label-away'">Run here</span>

    <span
      v-if="phase === 'waiting' || phase === 'done'"
      class="absolute inset-0 grid place-items-center"
      aria-hidden="true"
    >
      <svg class="size-4" viewBox="0 0 16 16" fill="none">
        <circle
          class="run-ring"
          :class="phase === 'waiting' ? 'run-ring-open' : 'run-ring-closed'"
          cx="8"
          cy="8"
          r="6.4"
          pathLength="100"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linecap="round"
        />
        <path
          v-if="phase === 'done'"
          class="run-check"
          d="M5.2 8.3 7.1 10.1 10.9 6"
          pathLength="1"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </span>

    <span class="sr-only" aria-live="polite">{{ phase === 'done' ? 'Session started' : '' }}</span>
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

.run-ring-closed {
  stroke-dasharray: 100;
  stroke-dashoffset: 0;
  animation: run-ring-close 340ms cubic-bezier(0.3, 0.7, 0.2, 1);
}

.run-check {
  stroke-dasharray: 1;
  stroke-dashoffset: 1;
  animation: run-check-draw 240ms cubic-bezier(0.3, 0.7, 0.2, 1) 220ms forwards;
}

.run-done {
  background: color-mix(in srgb, var(--color-safe) 12%, transparent);
  border-color: color-mix(in srgb, var(--color-safe) 30%, transparent);
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

@keyframes run-ring-close {
  from {
    stroke-dashoffset: 100;
    transform: rotate(-90deg);
  }
  to {
    stroke-dashoffset: 0;
    transform: rotate(0deg);
  }
}

@keyframes run-check-draw {
  to {
    stroke-dashoffset: 0;
  }
}
</style>
