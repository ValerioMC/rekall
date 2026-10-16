<script setup lang="ts">
import { computed } from 'vue'
import { arrivalAngle, departureAngle, midpointAlong, smoothPath } from '@/features/diagram/lib/edge-path'
import { textWidth } from '@/features/diagram/lib/text-wrap'
import { markerOf } from './edge-marker'
import type { Emphasis } from '@/features/diagram/lib/emphasis'
import type { PlacedEdge } from '@/features/diagram/lib/layout/graph-layout'
import type { GraphEdge } from '@/features/diagram/lib/scene/graph-scene'

/**
 * A line between two elements, drawn the way its Mermaid source says: solid, dashed, dotted or
 * thick, with whatever sits on each end (an arrow, a UML marker, an ER cardinality) and its label.
 */
const props = defineProps<{ placed: PlacedEdge; edge: GraphEdge; emphasis: Emphasis }>()

const LABEL_FONT = 11

const start = computed(() => markerOf(props.edge.startEnd))
const end = computed(() => markerOf(props.edge.endEnd))
const path = computed(() => smoothPath(props.placed.points, end.value?.inset ?? 0, start.value?.inset ?? 0))
const first = computed(() => props.placed.points[0] ?? { x: 0, y: 0 })
const last = computed(() => props.placed.points[props.placed.points.length - 1] ?? { x: 0, y: 0 })
const labelAt = computed(() => props.placed.labelAt ?? midpointAlong(props.placed.points))
const labelWidth = computed(() => (props.edge.label ? textWidth(props.edge.label, LABEL_FONT) + 14 : 0))
</script>

<template>
  <g class="graph-edge" :class="[`line-${edge.line}`, `is-${emphasis}`]">
    <path :d="path" class="edge-line" />
    <path
      v-if="start"
      :d="start.d"
      class="edge-marker"
      :class="`fill-${start.fill}`"
      :transform="`translate(${first.x} ${first.y}) rotate(${departureAngle(placed.points)})`"
    />
    <path
      v-if="end"
      :d="end.d"
      class="edge-marker"
      :class="`fill-${end.fill}`"
      :transform="`translate(${last.x} ${last.y}) rotate(${arrivalAngle(placed.points)})`"
    />
    <text v-if="edge.startLabel && placed.startLabelAt" :x="placed.startLabelAt.x + 6" :y="placed.startLabelAt.y - 6" class="edge-end-label">{{ edge.startLabel }}</text>
    <text v-if="edge.endLabel && placed.endLabelAt" :x="placed.endLabelAt.x + 6" :y="placed.endLabelAt.y - 6" class="edge-end-label">{{ edge.endLabel }}</text>
    <g v-if="edge.label && labelAt" class="edge-label" :transform="`translate(${labelAt.x} ${labelAt.y})`">
      <rect :x="-labelWidth / 2" y="-9.5" :width="labelWidth" height="19" rx="9.5" />
      <text text-anchor="middle" y="3.8">{{ edge.label }}</text>
    </g>
  </g>
</template>

<style scoped>
.graph-edge {
  --edge-tint: var(--color-text-subtle);
  transition: opacity var(--motion-quick) ease;
}

.edge-line {
  fill: none;
  stroke: var(--edge-tint);
  stroke-width: 1.4;
  stroke-linecap: round;
  transition:
    stroke var(--motion-quick) ease,
    stroke-width var(--motion-quick) ease;
}

.line-dashed .edge-line {
  stroke-dasharray: 6 4;
}

.line-dotted .edge-line {
  stroke-dasharray: 0.1 4.2;
  stroke-width: 2;
}

.line-thick .edge-line {
  stroke-width: 2.6;
}

.edge-marker {
  stroke: var(--edge-tint);
  stroke-width: 1.2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.fill-line {
  fill: var(--edge-tint);
  stroke: none;
}

.fill-canvas {
  fill: var(--color-canvas);
}

.fill-none {
  fill: none;
}

.edge-end-label {
  paint-order: stroke;
  stroke: var(--color-canvas);
  stroke-width: 4px;
  stroke-linejoin: round;
  font-family: var(--font-mono);
  font-size: var(--text-micro);
  fill: var(--color-text-muted);
}

.edge-label rect {
  fill: var(--color-surface);
  stroke: color-mix(in srgb, var(--edge-tint) 60%, transparent);
  stroke-width: 1;
}

.edge-label text {
  font-family: var(--font-sans);
  font-size: var(--text-caption);
  fill: var(--color-text-muted);
}

/* Lit: one step away from the selection, so it takes the selection's accent. */
.is-lit {
  --edge-tint: var(--color-accent-strong);
}

.is-lit .edge-line {
  stroke-width: 1.8;
}

.is-lit .edge-label text {
  fill: var(--color-text);
}

.is-dimmed {
  opacity: 0.12;
}
</style>
