<script setup lang="ts">
import { computed, ref } from 'vue'
import { smoothPath } from '@/features/diagram/lib/edge-path'
import { layoutFlow, placeSteps, type FlowLayout, type FlowShapeStep, type PlacedRoute } from '@/features/flow/lib/flow-layout'
import { FLOW_END, type RouteTarget } from '@/features/flow/lib/route-target'
import { useFlowDrag } from '@/features/flow/composables/useFlowDrag'
import { useFlowPan } from '@/features/flow/composables/useFlowPan'
import FlowTerminal from '@/features/flow/components/FlowTerminal.vue'
import type { Point } from '@/features/diagram/lib/viewport'
import type { ProjectConnection } from '@/features/connection/model/connection'
import type { DraftStep } from '@/features/graph/model/deployment-graph'
import type { ConnectionId } from '@/model/branded'
import GraphNode from './GraphNode.vue'

/**
 * A deployment graph drawn as it will run: Start, the steps top to bottom, End, and every route
 * labelled with its outcome. A step drags to where the person wants it, its routes following it;
 * picking one lights the routes that touch it, and its handle draws a new route. The empty canvas pans.
 */
const props = defineProps<{
  steps: readonly DraftStep[]
  bindings: readonly ProjectConnection[]
  selectedKey: string | null
}>()

const emit = defineEmits<{
  select: [key: string]
  connect: [from: string, to: RouteTarget<string>]
  place: [key: string, at: Point]
}>()

/** A route's id on the canvas: the step it leaves and its place among that step's routes. */
function routeId(step: DraftStep, index: number): string {
  return `${step.key}#${index}`
}

const shapes = computed<FlowShapeStep<string, string>[]>(() =>
  props.steps.map((step) => ({
    id: step.key,
    links: step.routes.map((route, index) => ({ id: routeId(step, index), outcome: route.outcome, toStepId: route.toKey }))
  }))
)
const base = computed((): FlowLayout<string, string> => layoutFlow(shapes.value))
const savedPlaces = computed(() => {
  const places = new Map<string, Point>()
  for (const step of props.steps) {
    if (step.flowX !== null && step.flowY !== null) places.set(step.key, { x: step.flowX, y: step.flowY })
  }
  return places
})
const outcomes = computed(() => {
  const all = new Map<string, string>()
  for (const step of props.steps) step.routes.forEach((route, index) => all.set(routeId(step, index), route.outcome))
  return all
})
const labelled = computed(() => layout.value.routes.filter((placed) => placed.kind === 'route'))
const bindingOf = computed(() => new Map<ConnectionId, ProjectConnection>(props.bindings.map((binding) => [binding.connectionId, binding])))

const surface = ref<HTMLElement | null>(null)
const scroller = ref<HTMLElement | null>(null)
const pan = useFlowPan(scroller, surface)
const drag = useFlowDrag<string>({
  surface,
  boxes: () => layout.value.nodes,
  endBox: () => layout.value.end,
  place: async (key, at) => emit('place', key, at),
  connect: (from, to) => emit('connect', from, to),
  select: (key) => emit('select', key)
})
const layout = computed(
  (): FlowLayout<string, string> => placeSteps(base.value, new Map([...savedPlaces.value, ...drag.live.value]))
)

function touchesSelection(placed: PlacedRoute<string, string>): boolean {
  return props.selectedKey !== null && (placed.from === props.selectedKey || placed.to === props.selectedKey)
}

/** A route into a step drawn above the one it leaves: the loop a failure takes. */
function loopsBack(placed: PlacedRoute<string, string>): boolean {
  const from = layout.value.nodes.get(placed.from as string)
  const to = layout.value.nodes.get(placed.to as string)
  return from !== undefined && to !== undefined && to.y < from.y
}

const pendingPath = computed(() => {
  const connecting = drag.connecting.value
  const from = connecting ? layout.value.nodes.get(connecting.from) : undefined
  if (!connecting || !from) return ''
  const start = { x: from.x + from.width / 2, y: from.y + from.height }
  const box = connecting.over === null ? null : connecting.over === FLOW_END ? layout.value.end : layout.value.nodes.get(connecting.over)
  const end = box ? { x: box.x + box.width / 2, y: box.y } : connecting.at
  return `M${start.x},${start.y}L${end.x},${end.y}`
})
</script>

<template>
  <div
    ref="scroller"
    class="texture-dots size-full min-h-0 overflow-auto"
    data-testid="graph-canvas"
    @pointerdown="pan.startPan($event)"
  >
    <div ref="surface" class="relative mx-auto" :style="{ width: `${layout.width}px`, height: `${layout.height}px` }">
      <svg
        class="pointer-events-none absolute inset-0 overflow-visible"
        :width="layout.width"
        :height="layout.height"
        aria-hidden="true"
      >
        <defs>
          <marker
            v-for="kind in ['rest', 'loop', 'focus']"
            :id="`graph-arrow-${kind}`"
            :key="kind"
            viewBox="-10 -5 10 10"
            refX="0"
            refY="0"
            markerWidth="9"
            markerHeight="9"
            orient="auto"
          >
            <path d="M0,0L-8.5,-4.4L-6.4,0L-8.5,4.4Z" :class="`graph-arrowhead-${kind}`" />
          </marker>
        </defs>
        <path
          v-for="placed in layout.routes"
          :key="placed.key"
          class="graph-route"
          :class="[touchesSelection(placed) && 'graph-route-focus', loopsBack(placed) && 'graph-route-loop']"
          :d="smoothPath(placed.points, 7)"
          :marker-end="`url(#graph-arrow-${touchesSelection(placed) ? 'focus' : loopsBack(placed) ? 'loop' : 'rest'})`"
        />
        <path v-if="pendingPath" :d="pendingPath" class="graph-pending" marker-end="url(#graph-arrow-focus)" />
      </svg>

      <span
        v-for="placed in labelled"
        :key="`label-${placed.key}`"
        class="graph-outcome pointer-events-none absolute -translate-x-1/2 -translate-y-1/2"
        :class="[touchesSelection(placed) && 'graph-route-focus', loopsBack(placed) && 'graph-route-loop']"
        :style="{ left: `${placed.labelAt.x}px`, top: `${placed.labelAt.y}px` }"
        data-testid="graph-outcome"
      >
        {{ placed.linkId ? outcomes.get(placed.linkId) : '' }}
      </span>

      <FlowTerminal kind="start" :box="layout.start" :reached="false" />
      <FlowTerminal kind="end" :box="layout.end" :reached="false" :aimed="drag.connecting.value?.over === FLOW_END" />

      <GraphNode
        v-for="(step, position) in steps"
        :key="step.key"
        :step="step"
        :position="position"
        :box="layout.nodes.get(step.key)!"
        :binding="step.connectionId ? (bindingOf.get(step.connectionId) ?? null) : null"
        :selected="step.key === selectedKey"
        :aimed="drag.connecting.value?.over === step.key"
        :dragging="drag.live.value.has(step.key)"
        @select="emit('select', step.key)"
        @move-start="drag.startMove(step.key, $event)"
        @connect-start="drag.startConnect(step.key, $event)"
        @nudge="(dx, dy) => drag.nudge(step.key, dx, dy)"
      />
    </div>
  </div>
</template>

<style scoped>
.graph-route {
  fill: none;
  stroke: var(--color-border-strong);
  stroke-width: 1.5;
  transition: stroke var(--motion-quick) ease;
}

.graph-route-loop {
  stroke: color-mix(in srgb, var(--color-warn) 45%, var(--color-border-strong));
  stroke-dasharray: 4 4;
}

.graph-route-focus {
  stroke: var(--color-accent);
  stroke-dasharray: none;
}

.graph-arrowhead-rest {
  fill: var(--color-border-strong);
}

.graph-arrowhead-loop {
  fill: color-mix(in srgb, var(--color-warn) 45%, var(--color-border-strong));
}

.graph-arrowhead-focus {
  fill: var(--color-accent);
}

.graph-pending {
  fill: none;
  stroke: var(--color-anchor);
  stroke-width: 1.75;
  stroke-dasharray: 5 4;
}

.graph-outcome {
  z-index: 2;
  border-radius: 9999px;
  border: 1px solid var(--color-border-strong);
  background: var(--color-surface);
  padding: 1px 8px;
  font-family: var(--font-mono);
  font-size: var(--text-caption);
  line-height: 18px;
  color: var(--color-text-muted);
  white-space: nowrap;
}

.graph-outcome.graph-route-loop {
  border-color: color-mix(in srgb, var(--color-warn) 40%, transparent);
  color: var(--color-warn);
}

.graph-outcome.graph-route-focus {
  border-color: color-mix(in srgb, var(--color-accent) 60%, transparent);
  color: var(--color-accent-strong);
}
</style>
