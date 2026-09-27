<script setup lang="ts">
import { computed, ref } from 'vue'
import { arrivalAngle, midpointAlong, smoothPath } from '@/common/diagram/edge-path'
import { humanise, relationFamily } from '@/common/diagram/kind-style'
import { textWidth } from '@/common/diagram/text-wrap'
import type { Emphasis } from '@/common/diagram/emphasis'
import type { PlacedEdge } from '@/common/diagram/graph-layout'
import type { ProjectedEdge } from '@/common/diagram/graph-projection'

/**
 * A relation. Flow is a solid line with a dart, the spine a reader follows; a condition is
 * dashed in the decision's colour and wears its condition; data flow is dotted in the data
 * colour; structure is a faint hairline with an open head. The relation's name appears on
 * hover only, because on every edge at once it is noise.
 */
const props = defineProps<{ placed: PlacedEdge; item: ProjectedEdge; emphasis: Emphasis }>()

const ARROW_INSET = 7
const LABEL_FONT = 11

const hovered = ref(false)

const family = computed(() => relationFamily(props.item.relation))
const path = computed(() => smoothPath(props.placed.points, ARROW_INSET))
const end = computed(() => props.placed.points[props.placed.points.length - 1] ?? { x: 0, y: 0 })
const angle = computed(() => arrivalAngle(props.placed.points))

const shownLabel = computed(() => {
  if (props.item.label) return props.item.label
  if (hovered.value || props.emphasis === 'lit') return humanise(props.item.relation)
  return null
})

const labelAt = computed(() => props.placed.labelAt ?? midpointAlong(props.placed.points))
const labelWidth = computed(() => (shownLabel.value ? textWidth(shownLabel.value, LABEL_FONT) + 14 : 0))
const merged = computed(() => props.item.edgeIds.length)
</script>

<template>
  <g
    class="diagram-edge"
    :class="[`family-${family}`, `is-${emphasis}`]"
    @pointerenter="hovered = true"
    @pointerleave="hovered = false"
  >
    <path :d="path" class="edge-hit" />
    <path :d="path" class="edge-line" />
    <path
      :d="family === 'structure' ? 'M-7,-4 L0,0 L-7,4' : 'M0,0 L-8.5,-4.4 L-6.4,0 L-8.5,4.4 Z'"
      class="edge-head"
      :class="{ 'is-open': family === 'structure' }"
      :transform="`translate(${end.x} ${end.y}) rotate(${angle})`"
    />
    <g v-if="shownLabel && labelAt" class="edge-label" :transform="`translate(${labelAt.x} ${labelAt.y})`">
      <rect :x="-labelWidth / 2" y="-9.5" :width="labelWidth" height="19" rx="9.5" />
      <text text-anchor="middle" y="3.8">
        {{ shownLabel }}<tspan v-if="merged > 1" class="edge-count"> ×{{ merged }}</tspan>
      </text>
    </g>
  </g>
</template>

<style scoped>
.diagram-edge {
  --edge-tint: var(--color-text-subtle);
  transition: opacity 160ms ease;
}

.family-conditional {
  --edge-tint: color-mix(in srgb, var(--color-kind-decision) 75%, var(--color-text-subtle));
}

.family-data {
  --edge-tint: color-mix(in srgb, var(--color-kind-data) 70%, var(--color-text-subtle));
}

.family-structure {
  --edge-tint: var(--color-border-strong);
}

.edge-hit {
  fill: none;
  stroke: transparent;
  stroke-width: 14;
}

.edge-line {
  fill: none;
  stroke: var(--edge-tint);
  stroke-width: 1.4;
  stroke-linecap: round;
  transition:
    stroke 140ms ease,
    stroke-width 140ms ease;
}

.family-conditional .edge-line {
  stroke-dasharray: 6 4;
}

.family-data .edge-line {
  stroke-dasharray: 0.1 4.2;
  stroke-width: 2;
}

.family-structure .edge-line {
  stroke-width: 1;
  stroke-dasharray: 3 3;
}

.edge-head {
  fill: var(--edge-tint);
  stroke: none;
}

.edge-head.is-open {
  fill: none;
  stroke: var(--edge-tint);
  stroke-width: 1.2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.edge-label rect {
  fill: var(--color-surface);
  stroke: color-mix(in srgb, var(--edge-tint) 60%, transparent);
  stroke-width: 1;
}

.edge-label text {
  font-family: var(--font-sans);
  font-size: 11px;
  fill: var(--color-text-muted);
}

.edge-count {
  fill: var(--color-text-subtle);
  font-family: var(--font-mono);
  font-size: 10px;
}

.diagram-edge:hover .edge-line {
  stroke-width: 2;
}

/* Lit: one relation away from the selection, so it takes the selection's accent. */
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
