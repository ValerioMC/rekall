<script setup lang="ts">
import { computed } from 'vue'
import NodeKindGlyph from '@/components/diagram/NodeKindGlyph.vue'
import { kindStyle } from '@/common/diagram/kind-style'
import type { GraphNode, NodeKind } from '@/model/diagram'

/**
 * The key to the drawing: every kind this diagram uses, with its count. Hovering a kind lights
 * its elements; clicking keeps them lit until clicked again.
 */
const props = defineProps<{ nodes: readonly GraphNode[]; pinned: NodeKind | null }>()

const emit = defineEmits<{ hover: [kind: NodeKind | null]; pin: [kind: NodeKind | null] }>()

const kinds = computed(() => {
  const counts = new Map<NodeKind, number>()
  for (const node of props.nodes) counts.set(node.kind, (counts.get(node.kind) ?? 0) + 1)
  return [...counts.entries()]
    .map(([kind, count]) => ({ kind, count, style: kindStyle(kind) }))
    .sort((a, b) => b.count - a.count || a.style.label.localeCompare(b.style.label))
})

const RELATIONS = [
  { family: 'flow', label: 'flow' },
  { family: 'conditional', label: 'condition' },
  { family: 'data', label: 'data' },
  { family: 'structure', label: 'depends' }
] as const
</script>

<template>
  <div class="glass w-[296px] rounded-[12px] border border-border p-1.5 shadow-lift" data-testid="diagram-legend">
    <ul class="grid grid-cols-2 gap-x-1" aria-label="Kinds">
      <li v-for="entry in kinds" :key="entry.kind">
        <button
          class="focus-ring flex h-6 w-full items-center gap-1.5 rounded-[6px] px-1.5 text-left text-[11.5px] transition-colors"
          :class="pinned === entry.kind ? 'bg-surface-raised text-text' : 'text-text-muted hover:bg-surface-raised/70 hover:text-text'"
          :aria-pressed="pinned === entry.kind"
          @pointerenter="emit('hover', entry.kind)"
          @pointerleave="emit('hover', null)"
          @focus="emit('hover', entry.kind)"
          @blur="emit('hover', null)"
          @click="emit('pin', pinned === entry.kind ? null : entry.kind)"
        >
          <span class="grid size-4 shrink-0 place-items-center" :style="{ color: entry.style.tint }">
            <NodeKindGlyph :glyph="entry.style.glyph" :size="14" />
          </span>
          <span class="min-w-0 flex-1 truncate">{{ entry.style.label }}</span>
          <span class="font-mono text-[10.5px] tabular-nums text-text-subtle">{{ entry.count }}</span>
        </button>
      </li>
    </ul>
    <div class="mt-1 grid grid-cols-4 gap-x-2 border-t border-border px-1.5 pb-0.5 pt-1.5" aria-label="Relations">
      <span v-for="relation in RELATIONS" :key="relation.family" class="flex items-center gap-1 text-[10.5px] text-text-subtle">
        <svg viewBox="0 0 16 6" class="h-1.5 w-4 shrink-0 overflow-visible" aria-hidden="true">
          <path d="M1 3h14" class="legend-line" :class="`family-${relation.family}`" />
        </svg>
        {{ relation.label }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.legend-line {
  fill: none;
  stroke-width: 1.4;
  stroke-linecap: round;
  stroke: var(--color-text-subtle);
}

.family-conditional {
  stroke: color-mix(in srgb, var(--color-kind-decision) 75%, var(--color-text-subtle));
  stroke-dasharray: 5 3;
}

.family-data {
  stroke: color-mix(in srgb, var(--color-kind-data) 70%, var(--color-text-subtle));
  stroke-dasharray: 0.1 3.6;
  stroke-width: 2;
}

.family-structure {
  stroke: var(--color-border-strong);
  stroke-dasharray: 3 3;
}
</style>
