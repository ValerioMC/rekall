<script setup lang="ts">
import type { NodeOutline } from '@/common/diagram/scene/graph-scene'
import type { PlacedNode } from '@/common/diagram/layout/graph-layout'
import type { Emphasis } from '@/common/diagram/emphasis'

/** A pseudo state or an unlabelled choice: a mark on the flow, not a card to read. */
defineProps<{ placed: PlacedNode; outline: NodeOutline; tint: string; emphasis: Emphasis }>()
</script>

<template>
  <g class="graph-mark" :class="`is-${emphasis}`" :style="{ '--tint': tint }" :transform="`translate(${placed.x} ${placed.y})`" aria-hidden="true">
    <circle v-if="outline === 'start'" :cx="placed.width / 2" :cy="placed.height / 2" :r="placed.width / 2" class="mark-solid" />
    <template v-else-if="outline === 'end'">
      <circle :cx="placed.width / 2" :cy="placed.height / 2" :r="placed.width / 2 - 0.7" class="mark-ring" />
      <circle :cx="placed.width / 2" :cy="placed.height / 2" :r="placed.width / 2 - 6" class="mark-solid" />
    </template>
    <rect v-else-if="outline === 'bar'" :width="placed.width" :height="placed.height" rx="3" class="mark-solid" />
    <path v-else :d="`M${placed.width / 2},0L${placed.width},${placed.height / 2}L${placed.width / 2},${placed.height}L0,${placed.height / 2}Z`" class="mark-ring" />
  </g>
</template>

<style scoped>
.graph-mark {
  transition: opacity var(--motion-quick) ease;
}

.mark-solid {
  fill: color-mix(in srgb, var(--tint) 70%, var(--color-text));
}

.mark-ring {
  fill: color-mix(in srgb, var(--tint) 10%, var(--color-canvas));
  stroke: color-mix(in srgb, var(--tint) 70%, var(--color-text));
  stroke-width: 1.4;
}

.is-dimmed {
  opacity: 0.22;
}
</style>
