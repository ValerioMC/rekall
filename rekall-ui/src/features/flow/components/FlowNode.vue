<script setup lang="ts">
import { computed } from 'vue'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { Box } from '@/features/diagram/lib/viewport'
import type { TaskStep } from '@/features/catalog/model/catalog'

/**
 * One step on the flow canvas: its number, seal and title and how many times the work has entered
 * it. The step the work stands on is lit. Dragged by its body to move it; its handle starts a route.
 */
const props = defineProps<{
  step: TaskStep
  box: Box
  current: boolean
  selected: boolean
  /** A route being drawn would land here. */
  aimed?: boolean
  /** The step is being dragged, so it floats above the others. */
  dragging?: boolean
}>()

const emit = defineEmits<{
  select: []
  'move-start': [event: PointerEvent]
  'connect-start': [event: PointerEvent]
  nudge: [dx: number, dy: number]
}>()

const NUDGE = 8
const ARROWS: Record<string, readonly [number, number]> = {
  ArrowLeft: [-NUDGE, 0],
  ArrowRight: [NUDGE, 0],
  ArrowUp: [0, -NUDGE],
  ArrowDown: [0, NUDGE]
}

/** Alt with an arrow moves the step, so plain arrows keep scrolling the canvas. */
function nudge(event: KeyboardEvent): void {
  const delta = event.altKey ? ARROWS[event.key] : undefined
  if (!delta) return
  event.preventDefault()
  emit('nudge', delta[0], delta[1])
}

const connections = useConnectionsStore()
const connection = computed(() => (props.step.connectionId ? connections.connectionById(props.step.connectionId) : null))

const visits = computed(() => props.step.visits.length)
const spent = computed(() => props.step.maxVisits !== null && visits.value >= props.step.maxVisits)
const number = computed(() => String(props.step.position + 1).padStart(2, '0'))

const plateClass = computed(() => {
  if (props.current) return 'panel-gold'
  if (props.step.state === 'CLAIMED') return 'panel-gold-quiet'
  if (props.step.state === 'DONE') return 'border-border bg-surface opacity-75'
  return 'panel-slate'
})

const label = computed(() => {
  const parts = [`Step ${props.step.position + 1}, ${props.step.title}`, props.step.state.toLowerCase()]
  if (props.current) parts.push('the work stands here')
  parts.push(visits.value === 1 ? 'entered once' : `entered ${visits.value} times`)
  if (props.step.maxVisits !== null) parts.push(`at most ${props.step.maxVisits}`)
  return parts.join(', ')
})
</script>

<template>
  <div
    class="group/node absolute"
    :class="dragging ? 'z-10' : 'z-[1]'"
    :style="{ left: `${box.x}px`, top: `${box.y}px`, width: `${box.width}px`, height: `${box.height}px` }"
  >
    <button
      type="button"
      class="focus-ring flex size-full touch-none cursor-grab select-none flex-col justify-between rounded-card border px-3 py-2.5 text-left transition-[border-color,box-shadow] duration-(--motion-quick) active:cursor-grabbing"
      :class="[
        plateClass,
        selected && 'flow-node-selected',
        aimed && 'flow-node-aimed',
        dragging && 'flow-node-dragging'
      ]"
      :aria-label="label"
      :aria-pressed="selected"
      :aria-keyshortcuts="'Alt+ArrowUp Alt+ArrowDown Alt+ArrowLeft Alt+ArrowRight'"
      :data-current="current || undefined"
      data-testid="flow-node"
      :data-step-id="step.id"
      @click="$emit('select')"
      @pointerdown="emit('move-start', $event)"
      @keydown="nudge"
    >
      <span class="flex min-w-0 items-start gap-2">
        <span class="mt-px shrink-0 font-mono text-micro font-semibold tabular-nums text-text-subtle">{{
          number
        }}</span>
        <span class="line-clamp-2 min-w-0 flex-1 text-small font-medium leading-snug text-text">{{
          step.title
        }}</span>
        <span
          class="relative mt-px block size-[18px] shrink-0"
          :class="step.state === 'RUNNING' && 'step-node-running rounded-full'"
          aria-hidden="true"
        >
          <StepSeal :state="step.state" :next="current && step.state === 'OPEN'" />
        </span>
      </span>

      <span class="flex items-center gap-1.5 text-micro">
        <span v-if="current" class="flow-tag text-accent" data-testid="flow-node-current">here</span>
        <span
          v-if="connection"
          class="flex min-w-0 items-center gap-1 font-mono text-text-muted"
          :title="`Reaches ${connection.title}`"
          data-testid="flow-node-connection"
        >
          <ConnectionKindIcon :kind="connection.kind" :size="11" />
          <span class="truncate">{{ connection.label }}</span>
        </span>
        <span
          class="ml-auto font-mono tabular-nums"
          :class="spent ? 'text-danger' : visits > 1 ? 'text-accent' : 'text-text-subtle'"
          data-testid="flow-node-visits"
        >
          <template v-if="step.maxVisits !== null">×{{ visits }}/{{ step.maxVisits }}</template>
          <template v-else-if="visits > 0">×{{ visits }}</template>
          <template v-else>not reached</template>
        </span>
      </span>
    </button>

    <span
      class="flow-handle absolute -bottom-2.5 left-1/2 grid size-5 -translate-x-1/2 touch-none cursor-crosshair place-items-center rounded-full border"
      :class="selected && 'flow-handle-shown'"
      title="Drag to another step, or to End, to add a route"
      aria-hidden="true"
      data-testid="flow-handle"
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
.flow-node-selected {
  box-shadow:
    0 0 0 2px var(--color-canvas),
    0 0 0 3.5px var(--color-anchor);
}

.flow-node-aimed {
  border-color: var(--color-anchor);
  box-shadow:
    0 0 0 2px var(--color-canvas),
    0 0 0 3.5px var(--color-anchor);
}

.flow-node-dragging {
  box-shadow: 0 10px 28px color-mix(in srgb, black 28%, transparent);
}

.flow-handle {
  border-color: var(--color-border-strong);
  background: var(--color-surface);
  color: var(--color-text-subtle);
  opacity: 0;
  transition:
    opacity var(--motion-quick) ease,
    color var(--motion-quick) ease,
    border-color var(--motion-quick) ease;
}

.group\/node:hover .flow-handle,
.group\/node:focus-within .flow-handle,
.flow-handle-shown {
  opacity: 1;
}

.flow-handle:hover {
  border-color: var(--color-anchor);
  color: var(--color-anchor-strong);
}

.flow-tag {
  border-radius: 9999px;
  border: 1px solid currentColor;
  padding: 0 6px;
  line-height: 15px;
  font-weight: 600;
  letter-spacing: 0.02em;
  opacity: 0.85;
}
</style>
