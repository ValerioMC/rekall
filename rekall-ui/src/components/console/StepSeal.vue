<script setup lang="ts">
/**
 * The mark a step wears on the checklist: one small instrument whose face is the step's state.
 *
 *   DRAFT    crop marks around a pencil: still being set, not on the list yet.
 *   OPEN     a hairline ring, the path not yet walked. The next one is a reticle: aimed at.
 *   RUNNING  the reticle comes alive. A comet orbits the bezel, each tick lights as it passes,
 *            and the core breathes on the pane's 1.7s beat.
 *   CLAIMED  a seal waiting for its stamp: the check is drawn but the ring is left open on the
 *            side facing the title, away from the rail that enters at twelve. Under the pointer
 *            the ring closes and the seal fills, which is what Accept does.
 *   DONE     the stamped seal, an engraved ring inside it. Green once the whole list is.
 *
 * Only a state change animates, and only once: a checklist loading in draws nothing, so a pane
 * at rest never moves unless a step is running.
 */
import { computed, onUnmounted, ref, watch } from 'vue'
import type { TaskStepState } from '@/model/catalog'

const props = withDefaults(
  defineProps<{
    state: TaskStepState
    /** The first open step: what the next session will pick up. */
    next?: boolean
    /** Every step on the checklist is done. */
    complete?: boolean
  }>(),
  { next: false, complete: false }
)

const TICK_COUNT = 12
const ORBIT_SECONDS = 3.4
/** Where the comet's head sits, clockwise from twelve, when its orbit starts. */
const HEAD_START_DEGREES = 80

interface Tick {
  readonly d: string
  readonly cardinal: boolean
  readonly delay: string
}

// Each tick peaks when the head crosses it, so the delay is the moment of that crossing.
const ticks: readonly Tick[] = Array.from({ length: TICK_COUNT }, (_, index) => {
  const degrees = (index * 360) / TICK_COUNT
  const radians = ((degrees - 90) * Math.PI) / 180
  const point = (radius: number): string =>
    `${(12 + radius * Math.cos(radians)).toFixed(2)} ${(12 + radius * Math.sin(radians)).toFixed(2)}`
  const crossing = ((((degrees - HEAD_START_DEGREES) / 360) % 1) + 1) % 1
  return {
    d: `M${point(6.7)}L${point(8.3)}`,
    cardinal: index % 3 === 0,
    delay: `${((crossing - 1) * ORBIT_SECONDS).toFixed(3)}s`
  }
})

const reticle = computed(() => props.state === 'RUNNING' || (props.state === 'OPEN' && props.next))
const visibleTicks = computed(() =>
  props.state === 'RUNNING' ? ticks : ticks.filter((tick) => tick.cardinal)
)
const showsCheck = computed(() => props.state !== 'RUNNING')

const arrival = ref<TaskStepState | null>(null)
let arrivalTimer: ReturnType<typeof setTimeout> | null = null

watch(
  () => props.state,
  (now, before) => {
    if (!before || now === before) return
    arrival.value = now
    if (arrivalTimer) clearTimeout(arrivalTimer)
    arrivalTimer = setTimeout(() => (arrival.value = null), 1100)
  }
)

onUnmounted(() => {
  if (arrivalTimer) clearTimeout(arrivalTimer)
})
</script>

<template>
  <svg
    class="seal"
    viewBox="0 0 24 24"
    fill="none"
    :data-state="state"
    :data-next="next ? 'true' : undefined"
    :data-complete="complete ? 'true' : undefined"
    :data-arrive="arrival ?? undefined"
    data-testid="step-seal"
    aria-hidden="true"
  >
    <g v-if="state === 'DRAFT'">
      <path
        class="seal-crop"
        d="M3.5 8.2V5a1.5 1.5 0 0 1 1.5-1.5h3.2M15.8 3.5H19A1.5 1.5 0 0 1 20.5 5v3.2M20.5 15.8V19a1.5 1.5 0 0 1-1.5 1.5h-3.2M8.2 20.5H5A1.5 1.5 0 0 1 3.5 19v-3.2"
      />
      <path class="seal-pencil" d="M14.2 7.4 16.6 9.8 10.6 15.8 7.4 16.6 8.2 13.4Z M12.9 8.7l2.4 2.4" />
      <path class="seal-stroke" pathLength="1" d="M7.4 18.5h9.2" />
    </g>

    <template v-else>
      <circle v-if="arrival === 'DONE'" class="seal-ripple" cx="12" cy="12" r="10.6" />
      <circle class="seal-disc" cx="12" cy="12" r="10.6" />
      <circle
        class="seal-ring"
        cx="12"
        cy="12"
        r="10.25"
        pathLength="100"
        :transform="state === 'CLAIMED' ? 'rotate(25 12 12)' : 'rotate(-90 12 12)'"
      />
      <circle v-if="state === 'DONE'" class="seal-engrave" cx="12" cy="12" r="8.3" />

      <g v-if="reticle" class="seal-ticks">
        <path
          v-for="tick in visibleTicks"
          :key="tick.d"
          :d="tick.d"
          :style="{ animationDelay: tick.delay }"
        />
      </g>

      <g v-if="state === 'RUNNING'" class="seal-comet">
        <circle class="seal-tail" cx="12" cy="12" r="10.25" pathLength="100" />
        <circle class="seal-head" cx="12" cy="12" r="10.25" pathLength="100" />
      </g>

      <circle v-if="reticle" class="seal-core" cx="12" cy="12" :r="state === 'RUNNING' ? 2.5 : 2.1" />

      <path
        v-if="showsCheck"
        class="seal-check"
        pathLength="1"
        d="M7.5 12.4 10.4 15.3 16.4 9.1"
      />
    </template>
  </svg>
</template>

<style scoped>
.seal {
  --seal: var(--color-accent);
  --seal-ink: var(--color-accent-ink);
  display: block;
  width: 100%;
  height: 100%;
  overflow: visible;
}

.seal[data-complete='true'] {
  --seal: var(--color-safe);
  --seal-ink: #03200f;
}

/* No transform in the shared transition: the ring's rotation is an SVG attribute that changes
   with the state, and easing between two rotations about a centre slides the ring off the disc. */
.seal circle,
.seal path {
  transition:
    fill 220ms ease,
    stroke 220ms ease,
    stroke-dasharray 320ms cubic-bezier(0.3, 0.7, 0.2, 1),
    opacity 200ms ease;
}

.seal-pencil,
.seal-ticks path,
.seal-core {
  transition:
    stroke 220ms ease,
    fill 220ms ease,
    opacity 200ms ease,
    transform 220ms ease;
}

/* What scales does so about its own centre. The ring and the comet keep the default origin:
   their rotation is an SVG attribute that already names its centre. */
.seal-disc,
.seal-ripple,
.seal-core,
.seal-pencil {
  transform-box: fill-box;
  transform-origin: center;
}

/* Ticks fold in toward the middle of the seal, not about their own midpoints. */
.seal-ticks path {
  transform-box: view-box;
  transform-origin: 12px 12px;
}

/* ---- Draft ---------------------------------------------------------------------------- */

.seal-crop {
  stroke: var(--color-border-strong);
  stroke-width: 1.2;
  stroke-linecap: round;
}

.seal-pencil {
  stroke: var(--color-text-subtle);
  stroke-width: 1.15;
  stroke-linejoin: round;
  stroke-linecap: round;
}

/* The line the nib has written: drawn only while the draft is under the pointer. */
.seal-stroke {
  stroke: var(--color-text-subtle);
  stroke-width: 1.1;
  stroke-linecap: round;
  stroke-dasharray: 1;
  stroke-dashoffset: 1;
  transition: stroke-dashoffset 420ms cubic-bezier(0.3, 0.7, 0.2, 1);
}

:is([data-seal-host]:hover, [data-seal-host]:focus-within) .seal-crop {
  stroke: var(--color-text-subtle);
}

:is([data-seal-host]:hover, [data-seal-host]:focus-within) .seal-pencil {
  stroke: var(--color-text-muted);
  transform: translate(-0.6px, -0.4px) rotate(-6deg);
}

:is([data-seal-host]:hover, [data-seal-host]:focus-within) .seal-stroke {
  stroke-dashoffset: 0;
}

/* ---- The disc and ring, shared by every checklist state ------------------------------ */

.seal-disc {
  fill: var(--color-canvas);
}

.seal-ring {
  stroke: var(--color-border-strong);
  stroke-width: 1.1;
  stroke-dasharray: 100 0;
  stroke-linecap: round;
}

.seal-check {
  stroke: var(--seal);
  stroke-width: 1.8;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-dasharray: 1;
  stroke-dashoffset: 0;
  opacity: 0;
}

/* ---- Open ----------------------------------------------------------------------------- */

/* The host (`data-seal-host`) is the button the seal sits in: hovering or focusing it previews
   what the click will do. */
:is([data-seal-host]:hover, [data-seal-host]:focus-visible) > .seal[data-state='OPEN'] .seal-check {
  opacity: 0.55;
}

[data-seal-host]:hover > .seal[data-state='OPEN'] .seal-ring {
  stroke: color-mix(in srgb, var(--seal) 80%, transparent);
}

.seal[data-state='OPEN'][data-next='true'] .seal-ring {
  stroke: var(--seal);
  stroke-width: 1.2;
}

.seal-ticks path {
  stroke: var(--seal);
  stroke-width: 1.1;
  stroke-linecap: round;
}

.seal-core {
  fill: var(--seal);
}

/* Aimed at, then fired: the reticle folds away under the pointer and the check takes its place. */
:is([data-seal-host]:hover, [data-seal-host]:focus-visible)
  > .seal[data-state='OPEN']
  :is(.seal-ticks path, .seal-core) {
  opacity: 0;
  transform: scale(0.4);
}

/* ---- Running -------------------------------------------------------------------------- */

.seal[data-state='RUNNING'] .seal-disc {
  fill: color-mix(in srgb, var(--seal) 14%, var(--color-canvas));
}

.seal[data-state='RUNNING'] .seal-ring {
  stroke: color-mix(in srgb, var(--seal) 34%, transparent);
  stroke-width: 1.2;
}

.seal[data-state='RUNNING'] .seal-ticks path {
  opacity: 0.32;
  animation: seal-tick 3.4s linear infinite;
}

.seal-comet {
  transform-box: view-box;
  transform-origin: 12px 12px;
  animation:
    seal-orbit 3.4s linear infinite,
    seal-fade-in 360ms ease-out both;
}

.seal-tail,
.seal-head {
  stroke-linecap: round;
  transform-box: view-box;
  transform-origin: 12px 12px;
  transform: rotate(-90deg);
}

.seal-tail {
  stroke: color-mix(in srgb, var(--seal) 70%, transparent);
  stroke-width: 1.6;
  stroke-dasharray: 25 75;
}

/* The head rides the leading end of the tail. */
.seal-head {
  stroke: var(--color-accent-strong);
  stroke-width: 2.3;
  stroke-dasharray: 5 95;
  stroke-dashoffset: -20;
  filter: drop-shadow(0 0 1.6px var(--seal));
}

.seal[data-state='RUNNING'] .seal-core {
  fill: var(--color-accent-strong);
  animation: dial-core 1.7s ease-in-out infinite;
}

/* ---- Claimed -------------------------------------------------------------------------- */

.seal[data-state='CLAIMED'] .seal-disc {
  fill: color-mix(in srgb, var(--seal) 13%, var(--color-canvas));
}

.seal[data-state='CLAIMED'] .seal-ring {
  stroke: var(--seal);
  stroke-width: 1.3;
  stroke-dasharray: 78 22;
}

.seal[data-state='CLAIMED'] .seal-check {
  opacity: 1;
  stroke-width: 1.7;
}

/* The stamp, previewed: the ring closes, the seal fills, the check turns to ink. */
:is([data-seal-host]:hover, [data-seal-host]:focus-visible) > .seal[data-state='CLAIMED'] .seal-ring {
  stroke-dasharray: 100 0;
}

:is([data-seal-host]:hover, [data-seal-host]:focus-visible) > .seal[data-state='CLAIMED'] .seal-disc {
  fill: var(--seal);
}

:is([data-seal-host]:hover, [data-seal-host]:focus-visible)
  > .seal[data-state='CLAIMED']
  .seal-check {
  stroke: var(--seal-ink);
  stroke-width: 2.1;
}

/* ---- Done ----------------------------------------------------------------------------- */

.seal[data-state='DONE'] .seal-disc {
  fill: var(--seal);
}

.seal[data-state='DONE'] .seal-ring {
  stroke: var(--seal);
}

.seal-engrave {
  stroke: var(--seal-ink);
  stroke-width: 0.6;
  opacity: 0.3;
}

.seal[data-state='DONE'] .seal-check {
  opacity: 1;
  stroke: var(--seal-ink);
  stroke-width: 2.1;
}

/* ---- Arrivals: one each, on a change of state ----------------------------------------- */

.seal[data-arrive='DONE'] .seal-disc {
  animation: seal-stamp 460ms cubic-bezier(0.2, 0.9, 0.3, 1.35);
}

.seal[data-arrive='DONE'] .seal-check,
.seal[data-arrive='CLAIMED'] .seal-check {
  animation: seal-draw 380ms cubic-bezier(0.4, 0, 0.2, 1) 160ms both;
}

.seal[data-arrive='DONE'] .seal-engrave {
  animation: seal-fade-in 500ms ease-out 260ms both;
}

.seal-ripple {
  stroke: var(--seal);
  stroke-width: 1.4;
  animation: seal-ripple 760ms cubic-bezier(0.2, 0.7, 0.3, 1) both;
}

.seal[data-arrive='CLAIMED'] .seal-ring {
  animation: seal-close-in 560ms cubic-bezier(0.3, 0.7, 0.2, 1) both;
}

@keyframes seal-orbit {
  to {
    transform: rotate(360deg);
  }
}

@keyframes seal-tick {
  0% {
    opacity: 1;
    stroke: var(--color-accent-strong);
  }
  30% {
    opacity: 0.45;
  }
  60%,
  100% {
    opacity: 0.32;
  }
}

@keyframes seal-fade-in {
  from {
    opacity: 0;
  }
}

@keyframes seal-stamp {
  0% {
    transform: scale(0.55);
  }
  60% {
    transform: scale(1.1);
  }
  100% {
    transform: scale(1);
  }
}

@keyframes seal-draw {
  from {
    stroke-dashoffset: 1;
  }
  to {
    stroke-dashoffset: 0;
  }
}

@keyframes seal-ripple {
  from {
    opacity: 0.7;
    transform: scale(1);
  }
  to {
    opacity: 0;
    transform: scale(1.9);
  }
}

@keyframes seal-close-in {
  from {
    stroke-dasharray: 0 100;
  }
  to {
    stroke-dasharray: 78 22;
  }
}
</style>
