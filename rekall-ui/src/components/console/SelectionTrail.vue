<script setup lang="ts">
/**
 * A gold comet that circles the selected row's edge so it can be found at a glance. Its tail is
 * stacked dashes of shrinking length sharing one head: every layer covers the head, only the
 * longest reaches the end, so the light is strongest at the front and fades along the tail.
 */
interface TrailLayer {
  /** Dash length as a share of the perimeter, in percent. */
  length: number
  opacity: number
  width: number
}

const LAYERS: readonly TrailLayer[] = [
  { length: 30, opacity: 0.16, width: 1.3 },
  { length: 22, opacity: 0.2, width: 1.3 },
  { length: 15, opacity: 0.26, width: 1.4 },
  { length: 9, opacity: 0.34, width: 1.5 },
  { length: 4.5, opacity: 0.5, width: 1.7 },
  { length: 1.6, opacity: 0.95, width: 1.9 }
]
</script>

<template>
  <svg class="select-trail" aria-hidden="true" data-testid="selection-trail">
    <rect
      v-for="layer in LAYERS"
      :key="layer.length"
      class="select-trail-dash"
      width="100%"
      height="100%"
      rx="5"
      pathLength="100"
      :stroke-width="layer.width"
      :stroke-opacity="layer.opacity"
      :stroke-dasharray="`${layer.length} 100`"
      :style="{ '--trail-length': layer.length }"
    />
  </svg>
</template>

<style scoped>
.select-trail {
  position: absolute;
  inset: 0.5px;
  width: calc(100% - 1px);
  height: calc(100% - 1px);
  overflow: visible;
  pointer-events: none;
}

.select-trail-dash {
  fill: none;
  stroke: var(--color-accent-strong);
  stroke-linecap: round;
  /* A dash spans [start, start + L]: lag each layer by L percent of the lap, less one lap so it
     is already running, and every layer ends on the same head. */
  animation: select-trail-orbit 3.2s calc(var(--trail-length) * 0.032s - 3.2s) linear infinite;
}

@keyframes select-trail-orbit {
  from {
    stroke-dashoffset: 0;
  }
  to {
    stroke-dashoffset: -100;
  }
}

@media (prefers-reduced-motion: reduce) {
  .select-trail {
    display: none;
  }
}
</style>
