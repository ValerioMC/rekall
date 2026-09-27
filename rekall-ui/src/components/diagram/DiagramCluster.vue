<script setup lang="ts">
import { computed } from 'vue'
import NodeKindGlyph from '@/components/diagram/NodeKindGlyph.vue'
import { kindStyle } from '@/common/diagram/kind-style'
import { CLUSTER_HEADER } from '@/common/diagram/graph-layout'
import { spanLabel } from '@/common/diagram/code-index'
import { textWidth } from '@/common/diagram/text-wrap'
import type { Emphasis } from '@/common/diagram/emphasis'
import type { PlacedCluster } from '@/common/diagram/graph-layout'
import type { ProjectedNode } from '@/common/diagram/graph-projection'

/**
 * A node drawn as the frame around what it contains: in the code lens, a function around the
 * conceptual pieces it implements. The header names it and where it lives, and folds it.
 */
const props = defineProps<{ placed: PlacedCluster; item: ProjectedNode; emphasis: Emphasis }>()

const emit = defineEmits<{ select: [id: string]; toggleFold: [id: string] }>()

const style = computed(() => kindStyle(props.item.node.kind))
const isCode = computed(() => style.value.shape === 'block')
const source = computed(() => props.item.node.sources[0] ?? null)
const where = computed(() => (source.value ? `${source.value.file} · ${spanLabel(source.value)}` : null))
/** The location is dropped rather than crushed when the frame is too narrow for both. */
const showWhere = computed(
  () => where.value !== null && textWidth(props.item.node.title, 12) + textWidth(where.value, 10.5) + 90 < props.placed.width
)
</script>

<template>
  <g
    class="diagram-cluster"
    :class="[`is-${emphasis}`, { 'is-code': isCode }]"
    :style="{ '--tint': style.tint }"
    :transform="`translate(${placed.x} ${placed.y})`"
    :data-node-id="item.id"
  >
    <rect :width="placed.width" :height="placed.height" rx="14" class="cluster-frame" />
    <path
      :d="`M0,14A14,14 0 0 1 14,0H${placed.width - 14}A14,14 0 0 1 ${placed.width},14V${CLUSTER_HEADER}H0Z`"
      class="cluster-band"
    />
    <g
      class="cluster-header"
      role="button"
      tabindex="0"
      :aria-label="`${style.label}: ${item.node.title}`"
      @click.stop="emit('select', item.id)"
      @keydown.enter.prevent="emit('select', item.id)"
    >
      <rect :width="placed.width - 36" :height="CLUSTER_HEADER" fill="transparent" />
      <NodeKindGlyph :glyph="style.glyph" :x="12" :y="8" :size="14" class="cluster-glyph" />
      <text x="33" y="19.5" class="cluster-title">{{ item.node.title }}</text>
      <text v-if="showWhere" :x="placed.width - 40" y="19" text-anchor="end" class="cluster-where">{{ where }}</text>
    </g>
    <g
      class="cluster-fold"
      :transform="`translate(${placed.width - 30} 5)`"
      role="button"
      tabindex="0"
      :aria-label="`Fold ${item.node.title}`"
      @click.stop="emit('toggleFold', item.id)"
      @keydown.enter.prevent="emit('toggleFold', item.id)"
    >
      <rect width="22" height="20" rx="6" />
      <path d="M7 12.5 11 8.5l4 4" />
    </g>
  </g>
</template>

<style scoped>
.diagram-cluster {
  transition: opacity 160ms ease;
}

.cluster-frame {
  fill: color-mix(in srgb, var(--tint) 3%, transparent);
  stroke: color-mix(in srgb, var(--tint) 24%, transparent);
  stroke-width: 1;
}

.cluster-band {
  fill: color-mix(in srgb, var(--tint) 7%, transparent);
}

.cluster-header {
  cursor: pointer;
  outline: none;
}

.cluster-glyph {
  color: var(--tint);
}

.cluster-title {
  font-family: var(--font-sans);
  font-size: 12px;
  font-weight: 600;
  fill: var(--color-text);
}

.is-code .cluster-title {
  font-family: var(--font-mono);
  font-weight: 500;
}

.cluster-where {
  font-family: var(--font-mono);
  font-size: 10.5px;
  fill: var(--color-text-subtle);
}

.cluster-fold {
  cursor: pointer;
  outline: none;
}

.cluster-fold rect {
  fill: transparent;
  transition: fill 140ms ease;
}

.cluster-fold path {
  fill: none;
  stroke: var(--color-text-subtle);
  stroke-width: 1.4;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.cluster-fold:hover rect,
.cluster-fold:focus-visible rect {
  fill: color-mix(in srgb, var(--tint) 16%, transparent);
}

.cluster-fold:hover path {
  stroke: var(--color-text);
}

.cluster-header:focus-visible .cluster-title {
  text-decoration: underline;
  text-decoration-color: var(--color-accent);
}

.is-selected .cluster-frame {
  stroke: var(--color-accent);
  stroke-width: 1.4;
}

.is-lit .cluster-frame {
  stroke: color-mix(in srgb, var(--tint) 55%, transparent);
}

.is-dimmed {
  opacity: 0.35;
}
</style>
