<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { smoothPath } from '@/common/diagram/edge-path'
import { layoutFlow, type PlacedRoute } from '@/common/flow/flow-layout'
import type { StepFlow } from '@/common/flow/step-flow'
import type { StepLink } from '@/model/catalog'
import type { StepLinkId, TaskStepId } from '@/model/branded'
import FlowNode from './FlowNode.vue'

/**
 * The flow drawn as a graph: steps laid out by dagre, routes between them labelled with their
 * outcome. Walked routes are solid, routes not yet taken dotted, and the route the work last
 * came in by carries a light toward the step it stands on.
 */
const props = defineProps<{
  flow: StepFlow
  selectedId: TaskStepId | null
}>()

defineEmits<{ select: [id: TaskStepId] }>()

const layout = computed(() => layoutFlow(props.flow.steps))
const linkById = computed(() => {
  const all = new Map<StepLinkId, StepLink>()
  for (const step of props.flow.steps) for (const link of step.links) all.set(link.id, link)
  return all
})

const reducedMotion =
  typeof window !== 'undefined' && (window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false)

function routeClass(placed: PlacedRoute): string {
  if (placed.id === props.flow.lastRouteId) return 'flow-route-live'
  if (props.flow.walked.has(placed.id)) return 'flow-route-walked'
  return 'flow-route-pending'
}

function touchesSelection(placed: PlacedRoute): boolean {
  const link = linkById.value.get(placed.id)
  return link !== undefined && (link.fromStepId === props.selectedId || link.toStepId === props.selectedId)
}

function isTerminal(id: TaskStepId): boolean {
  const step = props.flow.steps.find((candidate) => candidate.id === id)
  return props.flow.isFlow && step !== undefined && step.links.length === 0
}

const scroller = ref<HTMLElement | null>(null)

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
  <div
    ref="scroller"
    class="texture-dots relative min-h-0 min-w-0 flex-1 overflow-auto"
    data-testid="flow-canvas"
  >
    <div class="relative mx-auto" :style="{ width: `${layout.width}px`, height: `${layout.height}px` }">
      <svg
        class="pointer-events-none absolute inset-0 overflow-visible"
        :width="layout.width"
        :height="layout.height"
        aria-hidden="true"
      >
        <defs>
          <marker
            id="flow-arrow"
            viewBox="-10 -5 10 10"
            refX="0"
            refY="0"
            markerWidth="9"
            markerHeight="9"
            orient="auto"
          >
            <path d="M0,0L-8.5,-4.4L-6.4,0L-8.5,4.4Z" fill="context-stroke" />
          </marker>
        </defs>
        <g
          v-for="placed in layout.routes"
          :key="placed.id"
          :class="[routeClass(placed), touchesSelection(placed) && 'flow-route-focus']"
        >
          <path
            :id="`flow-route-${placed.id}`"
            class="flow-route"
            :d="smoothPath(placed.points, 7)"
            marker-end="url(#flow-arrow)"
          />
          <circle
            v-if="placed.id === flow.lastRouteId && !reducedMotion"
            :key="flow.trail.length"
            r="4"
            class="flow-spark"
          >
            <animateMotion dur="1.7s" repeatCount="indefinite" :path="smoothPath(placed.points, 7)" />
          </circle>
        </g>
      </svg>

      <span
        v-for="placed in layout.routes"
        :key="`label-${placed.id}`"
        class="flow-outcome pointer-events-none absolute -translate-x-1/2 -translate-y-1/2"
        :class="[routeClass(placed), touchesSelection(placed) && 'flow-route-focus']"
        :style="{ left: `${placed.labelAt.x}px`, top: `${placed.labelAt.y}px` }"
        data-testid="flow-outcome"
      >
        {{ linkById.get(placed.id)?.outcome }}
      </span>

      <FlowNode
        v-for="step in flow.steps"
        :key="step.id"
        :step="step"
        :box="layout.nodes.get(step.id)!"
        :current="step.id === flow.currentId"
        :selected="step.id === selectedId"
        :entry="step.id === flow.entryId"
        :terminal="isTerminal(step.id)"
        @select="$emit('select', step.id)"
      />
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
