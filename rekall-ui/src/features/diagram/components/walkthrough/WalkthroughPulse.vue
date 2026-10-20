<script setup lang="ts">
import { computed } from 'vue'
import type { BeatMotion } from '@/features/diagram/lib/walkthrough/walkthrough-beats'
import type { PlacedMessage } from '@/features/diagram/lib/layout/sequence-layout'

/**
 * The current beat travelling along its arrow: a lit token leaves the sender and lands on the
 * receiver, which answers with a ring. Under reduced motion it simply sits where it lands.
 */
const props = defineProps<{ placed: PlacedMessage; motion: BeatMotion }>()

const LOOP_WIDTH = 34
const LOOP_HEIGHT = 26

const travel = computed(() => {
  const { fromX, toX, y, self } = props.placed
  if (self) return { '--x0': `${fromX}px`, '--x1': `${fromX + LOOP_WIDTH}px`, '--y0': `${y}px`, '--y1': `${y + LOOP_HEIGHT}px` }
  return { '--x0': `${fromX}px`, '--x1': `${toX}px`, '--y0': `${y}px`, '--y1': `${y}px` }
})

const landing = computed(() => (props.placed.self ? { x: props.placed.fromX, y: props.placed.y + LOOP_HEIGHT } : { x: props.placed.toX, y: props.placed.y }))
</script>

<template>
  <g class="walkthrough-pulse" :class="[`motion-${motion}`, { 'is-self': placed.self }]" :style="travel" aria-hidden="true">
    <circle :cx="landing.x" :cy="landing.y" r="7" class="pulse-ring" />
    <g class="pulse-token">
      <circle r="9" class="pulse-glow" />
      <circle r="4.5" class="pulse-core" />
    </g>
  </g>
</template>

<style scoped>
.walkthrough-pulse {
  --pulse-tint: var(--color-accent);
  pointer-events: none;
}

.motion-reply {
  --pulse-tint: var(--color-kind-data);
}

.pulse-token {
  transform: translate(var(--x1), var(--y1));
  animation: pulse-travel 720ms cubic-bezier(0.4, 0, 0.2, 1) both;
}

.is-self .pulse-token {
  animation-name: pulse-loop;
}

.pulse-glow {
  fill: color-mix(in srgb, var(--pulse-tint) 28%, transparent);
}

.pulse-core {
  fill: var(--pulse-tint);
  stroke: var(--color-canvas);
  stroke-width: 1.5;
}

.pulse-ring {
  fill: none;
  stroke: var(--pulse-tint);
  stroke-width: 1.5;
  opacity: 0;
  transform-box: fill-box;
  transform-origin: center;
  animation: pulse-ring 900ms ease-out 640ms both;
}

@keyframes pulse-travel {
  from {
    transform: translate(var(--x0), var(--y0));
  }
  to {
    transform: translate(var(--x1), var(--y1));
  }
}

@keyframes pulse-loop {
  0% {
    transform: translate(var(--x0), var(--y0));
  }
  40% {
    transform: translate(var(--x1), var(--y0));
  }
  60% {
    transform: translate(var(--x1), var(--y1));
  }
  100% {
    transform: translate(var(--x0), var(--y1));
  }
}

@keyframes pulse-ring {
  from {
    opacity: 0.9;
    transform: scale(0.6);
  }
  to {
    opacity: 0;
    transform: scale(2.6);
  }
}

@media (prefers-reduced-motion: reduce) {
  .pulse-token,
  .is-self .pulse-token {
    animation: none;
  }

  .is-self .pulse-token {
    transform: translate(var(--x0), var(--y1));
  }

  .pulse-ring {
    animation: none;
  }
}
</style>
