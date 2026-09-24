<script setup lang="ts">
/**
 * The mark in front of a task row: the step seal's little sibling, one 14px instrument whose face
 * says where the task's work stands, so a list of amber dots stops meaning five different things.
 *
 *   RESTING   not in progress: a status-coloured dot on its own halo, the way every row was.
 *   WAITING   in progress, nothing handed in: a hollow amber ring around a pilot light. On, not
 *             doing anything. The part of the checklist already accepted is drawn as a green arc
 *             over the ring from twelve, so a half-reviewed task reads as half-closed.
 *   LIVE      a session or timer is on it: the ring goes faint, a comet orbits it and the core
 *             breathes. The only state that moves, and only for as long as it holds.
 *   LIVE_CLAIMED
 *             the timer runs and a claim waits for you: the same orbit, so it still reads as
 *             running, but the breathing core is replaced by the claimed check. Something is on,
 *             and it is your turn.
 *   CLAIMED   work handed in and waiting for you: the step seal's claimed face, shrunk. A tinted
 *             disc, an amber check, the ring left open on the side facing the title: a stamp not
 *             yet applied.
 *   ACCEPTED  every piece of work accepted: the stamped seal, a filled green disc with the check
 *             cut out of it. What is left to do is move the task to Done.
 *
 * Only a change of state animates, and only once: a list loading in draws nothing.
 */
import { onUnmounted, ref, watch } from 'vue'
import type { TaskStatus } from '@/model/catalog'
import type { TaskMarkState } from '@/model/task-mark'

const props = withDefaults(
  defineProps<{
    state: TaskMarkState
    status: TaskStatus
    /** Share of the checklist accepted, 0 to 1. */
    accepted?: number
    selected?: boolean
  }>(),
  { accepted: 0, selected: false }
)

const arrival = ref<TaskMarkState | null>(null)
let arrivalTimer: ReturnType<typeof setTimeout> | null = null

watch(
  () => props.state,
  (now, before) => {
    if (!before || now === before) return
    arrival.value = now
    if (arrivalTimer) clearTimeout(arrivalTimer)
    arrivalTimer = setTimeout(() => (arrival.value = null), 900)
  }
)

onUnmounted(() => {
  if (arrivalTimer) clearTimeout(arrivalTimer)
})
</script>

<template>
  <svg
    class="mark"
    viewBox="0 0 14 14"
    fill="none"
    :data-state="state"
    :data-status="status"
    :data-selected="selected ? 'true' : undefined"
    :data-arrive="arrival ?? undefined"
    data-testid="task-mark"
    aria-hidden="true"
  >
    <template v-if="state === 'RESTING'">
      <circle class="mark-halo" cx="7" cy="7" r="7" />
      <circle class="mark-dot" cx="7" cy="7" :r="selected ? 4 : 3.5" />
    </template>

    <template v-else>
      <circle v-if="arrival === 'ACCEPTED'" class="mark-ripple" cx="7" cy="7" r="6.4" />
      <circle class="mark-disc" cx="7" cy="7" r="6.4" />
      <circle
        class="mark-ring"
        cx="7"
        cy="7"
        r="5.9"
        pathLength="100"
        :transform="state === 'CLAIMED' ? 'rotate(25 7 7)' : 'rotate(-90 7 7)'"
      />
      <circle
        v-if="state === 'WAITING' && accepted > 0"
        class="mark-progress"
        cx="7"
        cy="7"
        r="5.9"
        pathLength="100"
        transform="rotate(-90 7 7)"
        :stroke-dasharray="`${(accepted * 100).toFixed(1)} 100`"
        data-testid="task-mark-progress"
      />
      <circle v-if="state === 'ACCEPTED'" class="mark-engrave" cx="7" cy="7" r="4.7" />

      <circle
        v-if="state === 'LIVE' || state === 'LIVE_CLAIMED'"
        class="mark-comet" cx="7" cy="7" r="5.9" pathLength="100" />
      <circle
        v-if="state === 'WAITING' || state === 'LIVE'"
        class="mark-core"
        cx="7"
        cy="7"
        :r="state === 'LIVE' ? 2 : 1.5"
      />

      <path
        v-if="state === 'CLAIMED' || state === 'ACCEPTED' || state === 'LIVE_CLAIMED'"
        class="mark-check"
        pathLength="1"
        d="M4.3 7.2 6.1 9 9.7 5.3"
      />
    </template>
  </svg>
</template>

<style scoped>
.mark {
  --mark: var(--color-accent);
  display: block;
  width: 100%;
  height: 100%;
  overflow: visible;
}

.mark[data-status='TODO'] {
  --mark: var(--color-text-subtle);
}

.mark[data-status='BLOCKED'] {
  --mark: var(--color-danger);
}

.mark[data-status='DONE'] {
  --mark: var(--color-safe);
}

/* No transform here: the ring's rotation is an SVG attribute that changes with the state, and
   easing between two rotations about a centre slides the ring off the disc. */
.mark circle,
.mark path {
  transition:
    fill 220ms ease,
    stroke 220ms ease,
    stroke-dasharray 320ms cubic-bezier(0.3, 0.7, 0.2, 1),
    opacity 200ms ease;
}

.mark-disc,
.mark-ripple,
.mark-core,
.mark-dot {
  transform-box: fill-box;
  transform-origin: center;
}

/* ---- Resting -------------------------------------------------------------------------- */

.mark-halo {
  fill: color-mix(in srgb, var(--mark) 20%, transparent);
}

.mark-dot {
  fill: var(--mark);
}

/* ---- The disc and ring every in-progress face shares ------------------------------------ */

.mark-disc {
  fill: transparent;
}

.mark-ring {
  stroke: color-mix(in srgb, var(--color-accent) 55%, transparent);
  stroke-width: 1.2;
  stroke-dasharray: 100 0;
  stroke-linecap: round;
}

.mark-core {
  fill: var(--color-accent);
}

.mark-check {
  stroke-width: 1.5;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-dasharray: 1;
  stroke-dashoffset: 0;
}

/* ---- Waiting -------------------------------------------------------------------------- */

.mark[data-state='WAITING'] .mark-disc {
  fill: color-mix(in srgb, var(--color-accent) 8%, transparent);
}

/* Accepted work is green wherever it shows: the arc here, the whole disc once it is all of it. */
.mark-progress {
  stroke: var(--color-safe);
  stroke-width: 1.4;
  stroke-linecap: round;
}

/* ---- Live ----------------------------------------------------------------------------- */

.mark[data-state='LIVE'] .mark-disc,
.mark[data-state='LIVE_CLAIMED'] .mark-disc {
  fill: color-mix(in srgb, var(--color-accent) 16%, transparent);
}

.mark[data-state='LIVE'] .mark-ring,
.mark[data-state='LIVE_CLAIMED'] .mark-ring {
  stroke: color-mix(in srgb, var(--color-accent) 28%, transparent);
}

.mark-comet {
  stroke: var(--color-accent-strong);
  stroke-width: 1.6;
  stroke-linecap: round;
  stroke-dasharray: 30 70;
  filter: drop-shadow(0 0 1.2px var(--color-accent));
  transform-box: view-box;
  transform-origin: 7px 7px;
  animation:
    mark-orbit 2.2s linear infinite,
    mark-fade-in 320ms ease-out both;
}

.mark[data-state='LIVE'] .mark-core {
  fill: var(--color-accent-strong);
  animation: dial-core 1.7s ease-in-out infinite;
}

/* ---- Live, with a claim waiting ----------------------------------------------------------- */

.mark[data-state='LIVE_CLAIMED'] .mark-check {
  stroke: var(--color-accent-strong);
  stroke-width: 1.6;
}

/* ---- Claimed -------------------------------------------------------------------------- */

.mark[data-state='CLAIMED'] .mark-disc {
  fill: color-mix(in srgb, var(--color-accent) 18%, transparent);
}

.mark[data-state='CLAIMED'] .mark-ring {
  stroke: var(--color-accent);
  stroke-width: 1.3;
  stroke-dasharray: 76 24;
}

.mark[data-state='CLAIMED'] .mark-check {
  stroke: var(--color-accent);
}

/* ---- Accepted ------------------------------------------------------------------------- */

.mark[data-state='ACCEPTED'] .mark-disc {
  fill: var(--color-safe);
}

.mark[data-state='ACCEPTED'] .mark-ring {
  stroke: var(--color-safe);
}

/* The step seal's done ink: the check is cut into the green, not drawn on top of it. */
.mark-engrave {
  stroke: #03200f;
  stroke-width: 0.5;
  opacity: 0.3;
}

.mark[data-state='ACCEPTED'] .mark-check {
  stroke: #03200f;
  stroke-width: 1.7;
}

/* ---- Selected: the ring firms up the way the resting halo gets its inset line ------------ */

.mark[data-selected='true'][data-state='WAITING'] .mark-ring {
  stroke: color-mix(in srgb, var(--color-accent) 80%, transparent);
}

/* ---- Arrivals: one each, on a change of state ----------------------------------------- */

.mark[data-arrive='CLAIMED'] .mark-ring {
  animation: mark-close-in 520ms cubic-bezier(0.3, 0.7, 0.2, 1) both;
}

.mark[data-arrive='CLAIMED'] .mark-check,
.mark[data-arrive='LIVE_CLAIMED'] .mark-check,
.mark[data-arrive='ACCEPTED'] .mark-check {
  animation: mark-draw 360ms cubic-bezier(0.4, 0, 0.2, 1) 140ms both;
}

.mark[data-arrive='ACCEPTED'] .mark-disc {
  animation: mark-stamp 440ms cubic-bezier(0.2, 0.9, 0.3, 1.35);
}

.mark-ripple {
  stroke: var(--color-safe);
  stroke-width: 1.2;
  animation: mark-ripple 700ms cubic-bezier(0.2, 0.7, 0.3, 1) both;
}

@keyframes mark-orbit {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

@keyframes mark-fade-in {
  from {
    opacity: 0;
  }
}

@keyframes mark-close-in {
  from {
    stroke-dasharray: 0 100;
  }
  to {
    stroke-dasharray: 76 24;
  }
}

@keyframes mark-draw {
  from {
    stroke-dashoffset: 1;
  }
  to {
    stroke-dashoffset: 0;
  }
}

@keyframes mark-stamp {
  0% {
    transform: scale(0.55);
  }
  60% {
    transform: scale(1.12);
  }
  100% {
    transform: scale(1);
  }
}

@keyframes mark-ripple {
  from {
    opacity: 0.7;
    transform: scale(1);
  }
  to {
    opacity: 0;
    transform: scale(1.9);
  }
}
</style>
