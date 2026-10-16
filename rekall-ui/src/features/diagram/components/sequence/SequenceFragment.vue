<script setup lang="ts">
import { computed } from 'vue'
import { textWidth } from '@/features/diagram/lib/text-wrap'
import type { PlacedFragment } from '@/features/diagram/lib/layout/sequence-layout'

/**
 * A `loop`, `alt`, `opt`, `par`, `critical`, `break` or `rect` block: a frame with its keyword in
 * a tab, its condition beside it, and a dashed rule before each `else` or `and`. The frame is
 * drawn behind the lifelines and the words above them, so a bar never crosses a condition.
 */
const props = defineProps<{ placed: PlacedFragment; layer: 'frame' | 'words' }>()

const tabWidth = computed(() => textWidth(props.placed.kind, 11) + 20)
</script>

<template>
  <g class="sequence-fragment" :class="`kind-${placed.kind}`" :transform="`translate(${placed.x} ${placed.y})`" aria-hidden="true">
    <template v-if="layer === 'frame'">
      <rect :width="placed.width" :height="placed.height" rx="8" class="fragment-frame" />
      <path v-for="section in placed.sections" :key="section.y" :d="`M0,${section.y - placed.y}H${placed.width}`" class="fragment-rule" />
    </template>
    <template v-else>
      <template v-if="placed.kind !== 'rect'">
        <path :d="`M0,8A8,8 0 0 1 8,0H${tabWidth}V14L${tabWidth - 8},22H0Z`" class="fragment-tab" />
        <text x="10" y="15" class="fragment-kind">{{ placed.kind }}</text>
        <text v-if="placed.label" :x="tabWidth + 10" y="15" class="fragment-label">[{{ placed.label }}]</text>
      </template>
      <template v-for="section in placed.sections" :key="section.y">
        <text v-if="section.label" x="10" :y="section.y - placed.y + 16" class="fragment-label">[{{ section.label }}]</text>
      </template>
    </template>
  </g>
</template>

<style scoped>
.sequence-fragment {
  --fragment-tint: var(--color-kind-state);
}

.kind-alt,
.kind-opt {
  --fragment-tint: var(--color-kind-decision);
}

.kind-critical,
.kind-break {
  --fragment-tint: var(--color-kind-event);
}

.kind-rect {
  --fragment-tint: var(--color-kind-external);
}

.fragment-frame {
  fill: color-mix(in srgb, var(--fragment-tint) 3%, transparent);
  stroke: color-mix(in srgb, var(--fragment-tint) 34%, transparent);
  stroke-width: 1;
}

.fragment-tab {
  fill: color-mix(in srgb, var(--fragment-tint) 14%, var(--color-canvas));
  stroke: color-mix(in srgb, var(--fragment-tint) 34%, transparent);
  stroke-width: 1;
}

.fragment-kind {
  font-family: var(--font-mono);
  font-size: var(--text-micro);
  font-weight: 600;
  fill: color-mix(in srgb, var(--fragment-tint) 80%, var(--color-text));
}

.fragment-label {
  paint-order: stroke;
  stroke: var(--color-canvas);
  stroke-width: 4px;
  stroke-linejoin: round;
  font-family: var(--font-sans);
  font-size: var(--text-caption);
  fill: var(--color-text-muted);
}

.fragment-rule {
  stroke: color-mix(in srgb, var(--fragment-tint) 34%, transparent);
  stroke-dasharray: 4 4;
}
</style>
