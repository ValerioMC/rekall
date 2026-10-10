<script setup lang="ts">
import type { Box } from '@/common/diagram/viewport'
import type { Emphasis } from '@/common/diagram/emphasis'

/** A note: folded paper with its text, read as an aside to what it sits beside. */
defineProps<{ box: Box; lines: readonly string[]; emphasis: Emphasis }>()

const FOLD = 10
</script>

<template>
  <g class="canvas-note" :class="`is-${emphasis}`" :transform="`translate(${box.x} ${box.y})`">
    <path :d="`M0,0H${box.width - FOLD}L${box.width},${FOLD}V${box.height}H0Z`" class="note-paper" />
    <path :d="`M${box.width - FOLD},0V${FOLD}H${box.width}`" class="note-fold" />
    <text v-for="(line, index) in lines" :key="index" x="12" :y="20 + index * 16" class="note-text">{{ line }}</text>
  </g>
</template>

<style scoped>
.canvas-note {
  transition: opacity var(--motion-quick) ease;
}

.note-paper {
  fill: color-mix(in srgb, var(--color-kind-custom) 9%, var(--color-surface));
  stroke: color-mix(in srgb, var(--color-kind-custom) 38%, transparent);
  stroke-width: 1;
}

.note-fold {
  fill: none;
  stroke: color-mix(in srgb, var(--color-kind-custom) 38%, transparent);
  stroke-width: 1;
}

.note-text {
  font-family: var(--font-sans);
  font-size: var(--text-caption);
  fill: var(--color-text-muted);
}

.is-dimmed {
  opacity: 0.3;
}
</style>
