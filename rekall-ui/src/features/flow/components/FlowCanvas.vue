<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { smoothPath } from '@/features/diagram/lib/edge-path'
import type { Point } from '@/features/diagram/lib/viewport'
import { layoutFlow, placeSteps, type FlowLayout, type PlacedRoute } from '@/features/flow/lib/flow-layout'
import { FLOW_END, type RouteTarget } from '@/features/flow/lib/route-target'
import { pillBox, placeGates } from '@/features/flow/lib/route-gate'
import {
  routeSessionMark,
  sessionMarkOf,
  sessionRibbonHeights,
  type RouteSessionMark
} from '@/features/flow/lib/session-marks'
import type { StepFlow } from '@/features/flow/lib/step-flow'
import { useFlowDrag } from '@/features/flow/composables/useFlowDrag'
import { useFlowPan } from '@/features/flow/composables/useFlowPan'
import type { StepLink } from '@/features/steps/model/task-step'
import type { StepLinkId, TaskStepId } from '@/model/branded'
import FlowNode from './FlowNode.vue'
import FlowTerminal from './FlowTerminal.vue'

/**
 * The flow drawn as a graph: Start, the steps laid out by dagre and End, with the routes between
 * them labelled by outcome. Walked routes are solid, routes not yet taken sit on a soft track,
 * and the route the work last came in by is heavier, lit and carries a light
 * running toward the step it stands on. Steps drag to where the person
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

const base = computed(() => layoutFlow(props.flow.steps, sessionRibbonHeights(props.flow)))
const savedPlaces = computed(() => {
  const places = new Map<TaskStepId, Point>()
  for (const step of props.flow.steps) {
    if (step.flowX !== null && step.flowY !== null) places.set(step.id, { x: step.flowX, y: step.flowY })
  }
  return places
})

const surface = ref<HTMLElement | null>(null)
const drag = useFlowDrag<TaskStepId>({
  surface,
  boxes: () => layout.value.nodes,
  endBox: () => layout.value.end,
  place: async (id, at) => emit('place', id, at),
  connect: (from, to) => emit('connect', from, to),
  select: (id) => emit('select', id)
})

const layout = computed((): FlowLayout => placeSteps(base.value, new Map([...savedPlaces.value, ...drag.live.value])))
const linkById = computed(() => {
  const all = new Map<StepLinkId, StepLink>()
  for (const step of props.flow.steps) for (const link of step.links) all.set(link.id, link)
  return all
})
const routeMarks = computed(() => {
  const marks = new Map<string, RouteSessionMark>()
  for (const placed of layout.value.routes) {
    const link = placed.linkId ? linkById.value.get(placed.linkId) : undefined
    const mark = link ? routeSessionMark(link, props.flow) : null
    if (mark) marks.set(placed.key, mark)
  }
  return marks
})
const gates = computed(() => {
  const asked = layout.value.routes.flatMap((placed) => {
    const mark = routeMarks.value.get(placed.key)
    return mark?.kind === 'gate' ? [{ placed, mark }] : []
  })
  const obstacles = [
    ...layout.value.nodes.values(),
    ...labelled.value.flatMap((placed) => {
      const outcome = placed.linkId ? linkById.value.get(placed.linkId)?.outcome : undefined
      return outcome ? [pillBox(placed.labelAt, outcome)] : []
    })
  ]
  const spots = placeGates(
    asked.map(({ placed, mark }) => ({ key: placed.key, points: placed.points, label: mark.label })),
    obstacles
  )
  return asked.map(({ placed, mark }) => ({ key: placed.key, at: spots.get(placed.key)!, mark }))
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

function isPending(placed: PlacedRoute): boolean {
  return !isLive(placed) && !isWalked(placed)
}

function routeClass(placed: PlacedRoute): string {
  if (isLive(placed)) return 'flow-route-live'
  if (isWalked(placed)) return 'flow-route-walked'
  return 'flow-route-pending'
}

const ROUTE_END_INSET = 15
const routePaths = computed(
  () => new Map(layout.value.routes.map((placed) => [placed.key, smoothPath(placed.points, ROUTE_END_INSET)]))
)

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
              refX="-6.4"
              refY="0"
              :markerWidth="kind === 'live' ? 17 : 14"
              :markerHeight="kind === 'live' ? 17 : 14"
              markerUnits="userSpaceOnUse"
              orient="auto"
            >
              <path d="M0,0L-8.5,-4.4L-6.4,0L-8.5,4.4Z" :class="`flow-arrowhead flow-arrowhead-${kind}`" />
            </marker>
          </defs>
          <g
            v-for="placed in layout.routes"
            :key="placed.key"
            :class="[
              routeClass(placed),
              touchesSelection(placed) && 'flow-route-focus',
              routeMarks.get(placed.key)?.kind === 'shared' && 'flow-route-shared'
            ]"
            :style="{ '--session-accent': routeMarks.get(placed.key)?.accent }"
          >
            <path v-if="isPending(placed)" class="flow-route-track" :d="routePaths.get(placed.key)" />
            <path
              :id="`flow-route-${placed.key}`"
              class="flow-route"
              :d="routePaths.get(placed.key)"
              :marker-end="`url(#flow-arrow-${arrowKind(placed)})`"
            >
              <title v-if="routeMarks.get(placed.key)?.kind === 'shared'">Same Claude session</title>
            </path>
            <path v-if="isLive(placed) && !reducedMotion" class="flow-route-run" :d="routePaths.get(placed.key)" />
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

        <span
          v-for="gate in gates"
          :key="`gate-${gate.key}`"
          class="flow-gate pointer-events-none absolute -translate-x-1/2 -translate-y-1/2"
          :style="{ left: `${gate.at.x}px`, top: `${gate.at.y}px`, '--session-accent': gate.mark.accent }"
          role="img"
          :aria-label="gate.mark.description"
          data-testid="flow-gate"
        >
          <svg viewBox="0 0 16 16" class="size-3 shrink-0" aria-hidden="true">
            <path
              d="M8 1.5 13.6 4.75v6.5L8 14.5l-5.6-3.25v-6.5Z"
              fill="var(--session-accent)"
              stroke="var(--session-accent)"
              stroke-width="1"
              stroke-linejoin="round"
            />
            <path v-if="gate.mark.opens" d="M6.6 5.6 10.4 8l-3.8 2.4Z" fill="var(--color-canvas)" />
            <path
              v-else
              d="M5.4 8h5M8.6 5.8 10.8 8l-2.2 2.2"
              fill="none"
              stroke="var(--color-canvas)"
              stroke-width="1.4"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          {{ gate.mark.label }}
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
          :session="sessionMarkOf(flow, step.id)"
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
  stroke-width: 2.5;
  stroke-linecap: round;
  stroke-linejoin: round;
  transition:
    stroke var(--motion-quick) ease,
    stroke-width var(--motion-quick) ease,
    opacity var(--motion-quick) ease;
}

.flow-route-track {
  fill: none;
  stroke: color-mix(in srgb, var(--color-border-field) 24%, transparent);
  stroke-width: 9;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.flow-route-pending .flow-route {
  stroke: var(--color-border-field);
}

.flow-route-walked .flow-route {
  stroke: var(--color-accent);
}

.flow-route-live .flow-route {
  stroke: var(--color-accent);
  stroke-width: 3.5;
  filter: drop-shadow(0 0 4px color-mix(in srgb, var(--color-accent) 60%, transparent));
}

.flow-route-shared .flow-route {
  pointer-events: stroke;
}

.flow-route-shared.flow-route-pending .flow-route {
  stroke: color-mix(in srgb, var(--session-accent) 60%, var(--color-border-field));
}

.flow-route-focus .flow-route {
  stroke: var(--color-anchor);
}

.flow-route-focus .flow-route-track {
  stroke: var(--color-anchor-soft);
}

.flow-route-run {
  fill: none;
  stroke: var(--color-accent-strong);
  stroke-width: 2;
  stroke-linecap: round;
  stroke-dasharray: 4 12;
  animation: flow-run 0.9s linear infinite;
}

@keyframes flow-run {
  to {
    stroke-dashoffset: -16;
  }
}

@media (prefers-reduced-motion: reduce) {
  .flow-route-run {
    animation: none;
  }
}

.flow-arrowhead {
  fill: var(--color-border-field);
}

.flow-arrowhead-walked,
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

.flow-gate {
  z-index: 3;
  display: flex;
  align-items: center;
  gap: 5px;
  border-radius: 9999px;
  border: 1.5px solid var(--session-accent);
  background: var(--color-surface-raised);
  padding: 1px 9px 1px 6px;
  font-family: var(--font-mono);
  font-size: var(--text-caption);
  font-weight: 600;
  line-height: 18px;
  color: var(--color-text);
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
