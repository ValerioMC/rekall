<script setup lang="ts">
import { computed } from 'vue'
import type { Box } from '@/features/diagram/lib/viewport'

/**
 * Where the flow begins or ends. Start always leads to the first step; End is where a route
 * lands to say the work is complete, and it is a drop target while a route is being drawn.
 */
const props = defineProps<{
  kind: 'start' | 'end'
  box: Box
  /** The work has been through Start, or has reached End. */
  reached: boolean
  /** A route being drawn would land here. */
  aimed?: boolean
}>()

const label = computed(() => {
  const name = props.kind === 'start' ? 'Start' : 'End'
  const parts = [`${name} of the flow`]
  if (props.reached) parts.push(props.kind === 'start' ? 'the work has begun' : 'the work reached the end')
  return parts.join(', ')
})
</script>

<template>
  <div
    class="flow-terminal absolute flex items-center justify-center gap-2 rounded-full border text-small font-semibold tracking-wide"
    :class="[reached ? 'flow-terminal-reached' : 'flow-terminal-idle', aimed && 'flow-terminal-aimed']"
    :style="{ left: `${box.x}px`, top: `${box.y}px`, width: `${box.width}px`, height: `${box.height}px` }"
    role="img"
    :aria-label="label"
    :data-testid="`flow-${kind}`"
  >
    <svg v-if="kind === 'start'" viewBox="0 0 16 16" class="size-3.5" aria-hidden="true">
      <path d="M4 2.5v11l9-5.5z" fill="currentColor" />
    </svg>
    <svg v-else viewBox="0 0 16 16" class="size-3.5" aria-hidden="true">
      <rect x="3" y="3" width="10" height="10" rx="2" fill="currentColor" />
    </svg>
    {{ kind === 'start' ? 'Start' : 'End' }}
  </div>
</template>

<style scoped>
.flow-terminal {
  transition:
    border-color var(--motion-quick) ease,
    box-shadow var(--motion-quick) ease,
    color var(--motion-quick) ease;
}

.flow-terminal-idle {
  border-style: dashed;
  border-color: var(--color-border-strong);
  background: var(--color-surface);
  color: var(--color-text-subtle);
}

.flow-terminal-reached {
  border-color: var(--color-accent);
  background: color-mix(in srgb, var(--color-accent) 12%, var(--color-surface));
  color: var(--color-accent-strong);
}

.flow-terminal-aimed {
  border-style: solid;
  border-color: var(--color-anchor);
  color: var(--color-anchor-strong);
  box-shadow:
    0 0 0 2px var(--color-canvas),
    0 0 0 3.5px var(--color-anchor);
}
</style>
