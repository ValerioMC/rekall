<script setup lang="ts">
import { computed } from 'vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import type { Box } from '@/features/diagram/lib/viewport'
import type { ProjectConnection } from '@/features/connection/model/connection'
import type { DraftStep } from '@/features/graph/model/deployment-graph'

/**
 * One step of a deployment graph on the canvas: its number, title and the connection it reaches.
 * A click picks it for the inspector; its handle draws a route to another step or to End.
 */
const props = defineProps<{
  step: DraftStep
  position: number
  box: Box
  /** The project's binding of the step's connection; null when it reaches none or one unbound. */
  binding: ProjectConnection | null
  selected: boolean
  aimed?: boolean
}>()

const emit = defineEmits<{ select: []; 'connect-start': [event: PointerEvent] }>()

const number = computed(() => String(props.position + 1).padStart(2, '0'))
const unbound = computed(() => props.step.connectionId !== null && props.binding === null)

const reach = computed(() => {
  if (props.binding) return props.binding.target ? `${props.binding.label} · ${props.binding.target}` : props.binding.label
  return unbound.value ? 'Not bound to this project' : 'No connection'
})

const label = computed(() => `Step ${props.position + 1}, ${props.step.title || 'untitled'}, ${reach.value}`)
</script>

<template>
  <div
    class="group/node absolute z-[1]"
    :style="{ left: `${box.x}px`, top: `${box.y}px`, width: `${box.width}px`, height: `${box.height}px` }"
  >
    <button
      type="button"
      class="focus-ring panel-slate flex size-full cursor-pointer select-none items-stretch gap-2.5 rounded-card border px-2.5 py-2 text-left transition-[border-color,box-shadow] duration-(--motion-quick)"
      :class="[selected && 'graph-node-selected', aimed && 'graph-node-aimed']"
      :aria-label="label"
      :aria-pressed="selected"
      data-testid="graph-node"
      :data-step-key="step.key"
      @click="emit('select')"
    >
      <span
        class="grid w-9 shrink-0 place-items-center rounded-control border border-border bg-canvas/60"
        :class="unbound && 'border-warn/50'"
        aria-hidden="true"
      >
        <ConnectionKindIcon v-if="binding" :kind="binding.kind" :size="18" />
        <svg v-else class="size-4 text-text-subtle" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="5.25" stroke="currentColor" stroke-width="1.3" stroke-dasharray="2.4 2.2" />
        </svg>
      </span>
      <span class="flex min-w-0 flex-1 flex-col justify-between">
        <span class="flex min-w-0 items-start gap-1.5">
          <span class="mt-px shrink-0 font-mono text-micro font-semibold tabular-nums text-text-subtle">{{ number }}</span>
          <span class="line-clamp-2 min-w-0 flex-1 text-small font-medium leading-snug text-text">
            {{ step.title || 'Untitled step' }}
          </span>
        </span>
        <span
          class="truncate font-mono text-micro"
          :class="unbound ? 'text-warn' : binding ? 'text-text-muted' : 'text-text-subtle'"
          data-testid="graph-node-reach"
        >
          {{ reach }}
        </span>
      </span>
    </button>

    <span
      class="graph-handle absolute -bottom-2.5 left-1/2 grid size-5 -translate-x-1/2 touch-none cursor-crosshair place-items-center rounded-full border"
      :class="selected && 'graph-handle-shown'"
      title="Drag to another step, or to End, to add a route"
      aria-hidden="true"
      data-testid="graph-handle"
      @pointerdown.stop.prevent="emit('connect-start', $event)"
    >
      <svg viewBox="0 0 12 12" class="size-2.5">
        <path
          d="M6 1.5v9M2 7l4 3.5L10 7"
          fill="none"
          stroke="currentColor"
          stroke-width="1.6"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </span>
  </div>
</template>

<style scoped>
.graph-node-selected,
.graph-node-aimed {
  border-color: var(--color-accent);
  box-shadow:
    0 0 0 2px var(--color-canvas),
    0 0 0 3.5px var(--color-accent);
}

.graph-node-aimed {
  border-color: var(--color-anchor);
  box-shadow:
    0 0 0 2px var(--color-canvas),
    0 0 0 3.5px var(--color-anchor);
}

.graph-handle {
  border-color: var(--color-border-strong);
  background: var(--color-surface);
  color: var(--color-text-subtle);
  opacity: 0;
  transition:
    opacity var(--motion-quick) ease,
    color var(--motion-quick) ease,
    border-color var(--motion-quick) ease;
}

.group\/node:hover .graph-handle,
.group\/node:focus-within .graph-handle,
.graph-handle-shown {
  opacity: 1;
}

.graph-handle:hover {
  border-color: var(--color-anchor);
  color: var(--color-anchor-strong);
}
</style>
