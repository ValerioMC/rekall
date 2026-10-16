<script setup lang="ts">
import { GROUP_HEADER, type PlacedGroup } from '@/features/diagram/lib/layout/graph-layout'
import type { Emphasis } from '@/features/diagram/lib/emphasis'

/**
 * A subgraph, a composite state or a namespace: a frame around what it holds, its title in a
 * band at the top. A composite state is an element too, so its band selects it.
 */
defineProps<{ placed: PlacedGroup; label: string; tint: string; linked: boolean; emphasis: Emphasis }>()

const emit = defineEmits<{ select: [key: string] }>()
</script>

<template>
  <g class="graph-group" :class="`is-${emphasis}`" :style="{ '--tint': tint }" :transform="`translate(${placed.x} ${placed.y})`">
    <rect :width="placed.width" :height="placed.height" rx="14" class="group-frame" />
    <path :d="`M0,14A14,14 0 0 1 14,0H${placed.width - 14}A14,14 0 0 1 ${placed.width},14V${GROUP_HEADER}H0Z`" class="group-band" />
    <g
      class="group-header"
      role="button"
      tabindex="0"
      :aria-label="`Group: ${label || placed.id}`"
      :data-element="placed.id"
      @click.stop="emit('select', placed.id)"
      @keydown.enter.prevent="emit('select', placed.id)"
    >
      <rect :width="placed.width" :height="GROUP_HEADER" fill="transparent" />
      <text x="14" y="19.5" class="group-title">{{ label }}</text>
      <g v-if="linked" :transform="`translate(${placed.width - 28} 8)`" class="group-link">
        <path d="M7 4.6 4.6 7 7 9.4M11 4.6 13.4 7 11 9.4" />
      </g>
    </g>
  </g>
</template>

<style scoped>
.graph-group {
  transition: opacity var(--motion-quick) ease;
}

.group-frame {
  fill: color-mix(in srgb, var(--tint) 3%, transparent);
  stroke: color-mix(in srgb, var(--tint) 24%, transparent);
  stroke-width: 1;
}

.group-band {
  fill: color-mix(in srgb, var(--tint) 7%, transparent);
}

.group-header {
  cursor: pointer;
  outline: none;
}

.group-title {
  font-family: var(--font-sans);
  font-size: var(--text-small);
  font-weight: 600;
  fill: var(--color-text);
}

.group-link path {
  fill: none;
  stroke: var(--tint);
  stroke-width: 1.3;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.group-header:focus-visible .group-title {
  text-decoration: underline;
  text-decoration-color: var(--color-accent);
}

.is-selected .group-frame {
  stroke: var(--color-accent);
  stroke-width: 1.4;
}

.is-lit .group-frame {
  stroke: color-mix(in srgb, var(--tint) 55%, transparent);
}

.is-dimmed {
  opacity: 0.35;
}
</style>
