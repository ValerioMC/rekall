<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { smoothPath } from '@/features/diagram/lib/edge-path'
import type { Point } from '@/features/diagram/lib/viewport'
import { layoutFlow, placeSteps, type PlacedRoute } from '@/features/flow/lib/flow-layout'
import { FLOW_END, type RouteTarget } from '@/features/flow/lib/route-target'
import type { StepFlow } from '@/features/flow/lib/step-flow'
import { useFlowDrag } from '@/features/flow/composables/useFlowDrag'
import { useFlowPan } from '@/features/flow/composables/useFlowPan'
import type { StepLink } from '@/features/catalog/model/catalog'
import type { StepLinkId, TaskStepId } from '@/model/branded'
import FlowNode from './FlowNode.vue'
import FlowTerminal from './FlowTerminal.vue'

/**
 * The flow drawn as a graph: Start, the steps laid out by dagre and End, with the routes between
 * them labelled by outcome. Walked routes are solid, routes not yet taken dotted, and the route the
 * work last came in by carries a light toward the step it stands on. Steps drag to where the person
 * wants them; the handle under a step draws a route to another step or to End. A press on the empty
 * canvas pans the graph.
 */
const props = defineProps<{
  flow: StepFlow
  selectedId: TaskStepId | null
}>()

const emit = defineEmits<{
  select: [id: TaskStepId]
  place: [id: TaskStepId, at: Point]
  connect: [from: TaskStepId, to: RouteTarget]
  tidy: []
}>()

const base = computed(() => layoutFlow(props.flow.steps))
const savedPlaces = computed(() => {
  const places = new Map<TaskStepId, Point>()
  for (const step of props.flow.steps) {
    if (step.flowX !== null && step.flowY !== null) places.set(step.id, { x: step.flowX, y: step.flowY })
  }
  return places
})

const surface = ref<HTMLElement | null>(null)
const drag = useFlowDrag({
  surface,
  boxes: () => layout.value.nodes,
  endBox: () => layout.value.end,
  place: async (id, at) => emit('place', id, at),
  connect: (from, to) => emit('connect', from, to),
  select: (id) => emit('select', id)
})

const layout = computed(() => placeSteps(base.value, new Map([...savedPlaces.value, ...drag.live.value])))
const linkById = computed(() => {
  const all = new Map<StepLinkId, StepLink>()
  for (const step of props.flow.steps) for (const link of step.links) all.set(link.id, link)
  return all
})
const labelled = computed(() => layout.value.routes.filter((placed) => placed.kind === 'route'))
const tidyable = computed(() => savedPlaces.value.size > 0)

const reducedMotion =
  typeof window !== 'undefined' && (window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false)

function isWalked(placed: PlacedRoute): boolean {
  if (placed.kind === 'entry') return props.flow.trail.length > 0
  if (placed.kind === 'exit') return props.flow.endedFromId === placed.from
  return placed.linkId !== null && props.flow.walked.has(placed.linkId)
}

function isLive(placed: PlacedRoute): boolean {
  if (placed.kind === 'exit') return props.flow.endedFromId === placed.from
  if (placed.kind === 'entry') return false
  return placed.linkId !== null && placed.linkId === props.flow.lastRouteId
}

function routeClass(placed: PlacedRoute): string {
  if (isLive(placed)) return 'flow-route-live'
  if (isWalked(placed)) return 'flow-route-walked'
  return 'flow-route-pending'
}

function touchesSelection(placed: PlacedRoute): boolean {
  return placed.from === props.selectedId || placed.to === props.selectedId
}

/** Each arrowhead is its own marker, because a marker cannot take the colour of the path that uses it. */
type ArrowKind = 'pending' | 'walked' | 'live' | 'focus'
const arrowKinds: readonly ArrowKind[] = ['pending', 'walked', 'live', 'focus']

function arrowKind(placed: PlacedRoute): ArrowKind {
  if (touchesSelection(placed)) return 'focus'
  if (isLive(placed)) return 'live'
  return isWalked(placed) ? 'walked' : 'pending'
}

/** The path the pointer draws from the handle of the step a route leaves. */
const pendingPath = computed(() => {
  const connecting = drag.connecting.value
  const from = connecting ? layout.value.nodes.get(connecting.from) : undefined
  if (!connecting || !from) return ''
  const start = { x: from.x + from.width / 2, y: from.y + from.height }
  const end = connecting.over === null ? connecting.at : snapPoint(connecting.over)
  return `M${start.x},${start.y}L${end.x},${end.y}`
})

function snapPoint(target: RouteTarget): Point {
  const box = target === FLOW_END ? layout.value.end : layout.value.nodes.get(target)!
  return { x: box.x + box.width / 2, y: box.y }
}

const scroller = ref<HTMLElement | null>(null)
const pan = useFlowPan(scroller, surface)

/** Keeps the step the work stands on in view as the flow moves. */
async function revealCurrent(): Promise<void> {
  await nextTick()
  const box = props.flow.currentId ? layout.value.nodes.get(props.flow.currentId) : undefined
  const element = scroller.value
  if (!box || !element) return
  const left = box.x + box.width / 2 - element.clientWidth / 2
  const top = box.y + box.height / 2 - element.clientHeight / 2
  element.scrollTo?.({ left, top, behavior: reducedMotion ? 'auto' : 'smooth' })
}

watch(() => props.flow.currentId, revealCurrent)
onMounted(revealCurrent)
</script>

<template>
  <div class="relative min-h-0 min-w-0 flex-1">
    <button
      v-if="tidyable"
      type="button"
      class="key-quiet focus-ring absolute right-3 top-3 z-20 cursor-pointer rounded-control px-2.5 py-1 text-caption"
      data-testid="flow-tidy"
      @click="emit('tidy')"
    >
      Tidy layout
    </button>
    <div
      ref="scroller"
      class="texture-dots size-full overflow-auto"
      data-testid="flow-canvas"
      @pointerdown="pan.startPan($event)"
    >
      <div
        ref="surface"
        class="relative mx-auto"
        :style="{ width: `${layout.width}px`, height: `${layout.height}px` }"
      >
        <svg
          class="pointer-events-none absolute inset-0 overflow-visible"
          :width="layout.width"
          :height="layout.height"
          aria-hidden="true"
        >
          <defs>
            <marker
              v-for="kind in arrowKinds"
              :id="`flow-arrow-${kind}`"
              :key="kind"
              viewBox="-10 -5 10 10"
              refX="0"
              refY="0"
              markerWidth="9"
              markerHeight="9"
              orient="auto"
            >
              <path d="M0,0L-8.5,-4.4L-6.4,0L-8.5,4.4Z" :class="`flow-arrowhead flow-arrowhead-${kind}`" />
            </marker>
          </defs>
          <g
            v-for="placed in layout.routes"
            :key="placed.key"
            :class="[routeClass(placed), touchesSelection(placed) && 'flow-route-focus']"
          >
            <path
              :id="`flow-route-${placed.key}`"
              class="flow-route"
              :d="smoothPath(placed.points, 7)"
              :marker-end="`url(#flow-arrow-${arrowKind(placed)})`"
            />
            <circle v-if="isLive(placed) && !reducedMotion" :key="flow.trail.length" r="4" class="flow-spark">
              <animateMotion dur="1.7s" repeatCount="indefinite" :path="smoothPath(placed.points, 7)" />
            </circle>
          </g>
          <path
            v-if="pendingPath"
            :d="pendingPath"
            class="flow-pending"
            marker-end="url(#flow-arrow-focus)"
          />
        </svg>

        <span
          v-for="placed in labelled"
          :key="`label-${placed.key}`"
          class="flow-outcome pointer-events-none absolute -translate-x-1/2 -translate-y-1/2"
          :class="[routeClass(placed), touchesSelection(placed) && 'flow-route-focus']"
          :style="{ left: `${placed.labelAt.x}px`, top: `${placed.labelAt.y}px` }"
          data-testid="flow-outcome"
        >
          {{ placed.linkId ? linkById.get(placed.linkId)?.outcome : '' }}
        </span>

        <FlowTerminal kind="start" :box="layout.start" :reached="flow.trail.length > 0" />
        <FlowTerminal
          kind="end"
          :box="layout.end"
          :reached="flow.finished"
          :aimed="drag.connecting.value?.over === FLOW_END"
        />

        <FlowNode
          v-for="step in flow.steps"
          :key="step.id"
          :step="step"
          :box="layout.nodes.get(step.id)!"
          :current="step.id === flow.currentId"
          :selected="step.id === selectedId"
          :aimed="drag.connecting.value?.over === step.id"
          :dragging="drag.live.value.has(step.id)"
          @select="$emit('select', step.id)"
          @move-start="drag.startMove(step.id, $event)"
          @connect-start="drag.startConnect(step.id, $event)"
          @nudge="(dx, dy) => drag.nudge(step.id, dx, dy)"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.flow-route {
  fill: none;
  stroke-width: 1.5;
  transition:
    stroke var(--motion-quick) ease,
    opacity var(--motion-quick) ease;
}

.flow-route-pending .flow-route {
  stroke: var(--color-border-strong);
  stroke-dasharray: 3 4;
}

.flow-route-walked .flow-route {
  stroke: color-mix(in srgb, var(--color-accent) 55%, var(--color-border-strong));
}

.flow-route-live .flow-route {
  stroke: var(--color-accent);
  stroke-width: 2.25;
  filter: drop-shadow(0 0 4px color-mix(in srgb, var(--color-accent) 60%, transparent));
}

.flow-route-focus .flow-route {
  stroke: var(--color-anchor);
  stroke-dasharray: none;
}

.flow-arrowhead {
  fill: var(--color-border-strong);
}

.flow-arrowhead-walked {
  fill: color-mix(in srgb, var(--color-accent) 55%, var(--color-border-strong));
}

.flow-arrowhead-live {
  fill: var(--color-accent);
}

.flow-arrowhead-focus {
  fill: var(--color-anchor);
}

.flow-pending {
  fill: none;
  stroke: var(--color-anchor);
  stroke-width: 1.75;
  stroke-dasharray: 5 4;
}

.flow-spark {
  fill: var(--color-accent-strong);
  filter: drop-shadow(0 0 5px var(--color-accent));
}

.flow-outcome {
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

.flow-outcome.flow-route-walked,
.flow-outcome.flow-route-live {
  border-color: color-mix(in srgb, var(--color-accent) 45%, transparent);
  color: var(--color-accent-strong);
}

.flow-outcome.flow-route-focus {
  border-color: var(--color-anchor-line);
  color: var(--color-anchor-strong);
}
</style>
