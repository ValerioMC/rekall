<script setup lang="ts">
/**
 * The run queue's face in the top bar: one 22px dial whose face is the queue's state, drawn to
 * sit beside the usage ring as its sibling, because the two answer one question between them:
 * what Claude is doing on this machine and how much room is left to do it in.
 *
 *   EMPTY      three bars stepping down: a queue with nothing in it.
 *   READY      the same bars, the top one amber: something is lined up and nothing is armed.
 *   SCHEDULED  a clock face whose single hand points at the start time on a twelve-hour dial,
 *              so "it starts at two" is readable before the label is. Still: waiting is not work.
 *   RUNNING    a comet orbits the bezel and the core breathes, on the step seal's beat. The only
 *              state that moves, and only while a session is actually working.
 *   HOLDING    the ring filled to the ceiling and stopped there, with a pause mark in its core:
 *              the queue reached the line it was given and waits for the window to reset.
 *
 * A change of state lands once, as a small settle of the whole dial; a dial loading in draws
 * nothing, and re-rendering in the same state never replays it.
 */
import { computed, onUnmounted, ref, watch } from 'vue'
import type { RunDialFace } from '@/model/runQueue'

const props = withDefaults(
  defineProps<{
    face: RunDialFace
    /** SCHEDULED: when it starts, for the hand. */
    startAt?: string | null
    /** HOLDING: the ceiling, for how far the ring is filled. */
    ceilingPercent?: number | null
  }>(),
  { startAt: null, ceilingPercent: null }
)

const RADIUS = 8
const CIRCUMFERENCE = 2 * Math.PI * RADIUS
const COMET_ARC = CIRCUMFERENCE * 0.26
const ARRIVAL_MS = 340

/** Degrees clockwise from twelve for the start time on a twelve-hour face. */
const handAngle = computed(() => {
  if (!props.startAt) return 0
  const at = new Date(props.startAt)
  if (Number.isNaN(at.getTime())) return 0
  return ((at.getHours() % 12) + at.getMinutes() / 60) * 30
})

const holdOffset = computed(() => {
  const percent = Math.min(100, Math.max(0, props.ceilingPercent ?? 100))
  return CIRCUMFERENCE * (1 - percent / 100)
})

const CARDINALS: readonly string[] = [0, 90, 180, 270].map((degrees) => {
  const radians = ((degrees - 90) * Math.PI) / 180
  const point = (radius: number): string =>
    `${(11 + radius * Math.cos(radians)).toFixed(2)} ${(11 + radius * Math.sin(radians)).toFixed(2)}`
  return `M${point(6)}L${point(7.6)}`
})

const arriving = ref(false)
let arrivalTimer: ReturnType<typeof setTimeout> | undefined

watch(
  () => props.face,
  (next, previous) => {
    if (next === previous) return
    arriving.value = false
    clearTimeout(arrivalTimer)
    requestAnimationFrame(() => {
      arriving.value = true
      arrivalTimer = setTimeout(() => (arriving.value = false), ARRIVAL_MS)
    })
  }
)

onUnmounted(() => clearTimeout(arrivalTimer))
</script>

<template>
  <span
    class="relative grid size-[22px] shrink-0 place-items-center"
    :class="{ settle: arriving }"
    :data-face="face"
    data-testid="run-dial"
    aria-hidden="true"
  >
    <svg class="size-[22px]" viewBox="0 0 22 22" fill="none">
      <template v-if="face === 'empty' || face === 'ready'">
        <path
          d="M5 7h12"
          stroke-width="1.7"
          stroke-linecap="round"
          :class="face === 'ready' ? 'stroke-accent' : 'stroke-text-subtle'"
        />
        <path d="M5 11h9" class="stroke-text-subtle" stroke-width="1.7" stroke-linecap="round" />
        <path d="M5 15h6" class="stroke-text-subtle" stroke-width="1.7" stroke-linecap="round" />
      </template>

      <template v-else-if="face === 'scheduled'">
        <circle cx="11" cy="11" :r="RADIUS" class="stroke-border-strong" stroke-width="1.5" />
        <path
          v-for="(tick, index) in CARDINALS"
          :key="index"
          :d="tick"
          class="stroke-text-subtle"
          stroke-width="1.2"
          stroke-linecap="round"
        />
        <!-- Rotated by attribute, so no CSS transform or transform-origin may touch it. -->
        <path
          d="M11 11V5.2"
          class="stroke-accent"
          stroke-width="1.8"
          stroke-linecap="round"
          :transform="`rotate(${handAngle} 11 11)`"
          data-testid="run-dial-hand"
        />
        <circle cx="11" cy="11" r="1.5" class="fill-accent" />
      </template>

      <template v-else-if="face === 'running'">
        <circle cx="11" cy="11" :r="RADIUS" class="stroke-border" stroke-width="2" />
        <circle
          cx="11"
          cy="11"
          :r="RADIUS"
          class="dial-comet stroke-accent"
          stroke-width="2"
          stroke-linecap="round"
          :stroke-dasharray="`${COMET_ARC} ${CIRCUMFERENCE}`"
        />
        <circle cx="11" cy="11" r="2.6" class="dial-core fill-accent" />
      </template>

      <template v-else>
        <circle cx="11" cy="11" :r="RADIUS" class="stroke-border" stroke-width="2" />
        <circle
          cx="11"
          cy="11"
          :r="RADIUS"
          class="stroke-warn"
          stroke-width="2"
          stroke-linecap="round"
          :stroke-dasharray="CIRCUMFERENCE"
          :stroke-dashoffset="holdOffset"
          transform="rotate(-90 11 11)"
        />
        <path d="M9.4 8.6v4.8M12.6 8.6v4.8" class="stroke-warn" stroke-width="1.6" stroke-linecap="round" />
      </template>
    </svg>
  </span>
</template>

<style scoped>
/* One orbit every 1.7s, the beat the running step seal and the live-step rail already keep. */
.dial-comet {
  transform-origin: 11px 11px;
  animation: dial-orbit 1.7s linear infinite;
}

.dial-core {
  transform-box: fill-box;
  transform-origin: center;
  animation: dial-breathe 1.7s ease-in-out infinite;
}

@keyframes dial-orbit {
  from {
    transform: rotate(-90deg);
  }
  to {
    transform: rotate(270deg);
  }
}

@keyframes dial-breathe {
  0%,
  100% {
    transform: scale(0.8);
    opacity: 0.75;
  }
  50% {
    transform: scale(1.1);
    opacity: 1;
  }
}
</style>
