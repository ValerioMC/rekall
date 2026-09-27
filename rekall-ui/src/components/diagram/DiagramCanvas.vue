<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, useId, watch } from 'vue'
import DiagramCluster from '@/components/diagram/DiagramCluster.vue'
import DiagramEdge from '@/components/diagram/DiagramEdge.vue'
import DiagramNode from '@/components/diagram/DiagramNode.vue'
import { useDiagramCamera } from '@/composables/useDiagramCamera'
import { centreOn, isInView, panBy, zoomAt } from '@/common/diagram/viewport'
import { edgeEmphasis, nodeEmphasis, type Spotlight } from '@/common/diagram/emphasis'
import type { DiagramLayout } from '@/common/diagram/graph-layout'
import type { Projection } from '@/common/diagram/graph-projection'

/**
 * The drawing surface: an SVG world under one camera. Plain wheel or two fingers pan, pinch or
 * ⌘/Ctrl + wheel zooms around the pointer, dragging the background pans, a click on it clears
 * the selection. It draws what it is given and decides nothing about the graph.
 */
const props = defineProps<{
  diagramKey: string
  projection: Projection
  layout: DiagramLayout
  spotlight: Spotlight | null
}>()

const emit = defineEmits<{ select: [id: string | null]; toggleFold: [id: string] }>()

const { camera, viewport, set, animateTo } = useDiagramCamera()
const host = ref<HTMLElement | null>(null)
const patternId = `diagram-dots-${useId()}`

const clusters = computed(() =>
  [...props.layout.clusters.values()]
    .sort((a, b) => a.depth - b.depth)
    .map((placed) => ({ placed, item: props.projection.nodes.find((node) => node.id === placed.id)! }))
    .filter((cluster) => cluster.item !== undefined)
)

const nodes = computed(() =>
  props.projection.nodes
    .map((item) => ({ item, placed: props.layout.nodes.get(item.id) }))
    .filter((node): node is { item: (typeof props.projection.nodes)[number]; placed: NonNullable<typeof node.placed> } => node.placed !== undefined)
)

const edges = computed(() => {
  const byKey = new Map(props.projection.edges.map((edge) => [edge.key, edge]))
  return props.layout.edges
    .map((placed) => ({ placed, item: byKey.get(placed.key)! }))
    .filter((edge) => edge.item !== undefined)
})

/** Nodes arrive in reading order: a stagger along the flow, capped so a big diagram is not slow. */
const arriveSpan = computed(() => {
  const xs = nodes.value.map((node) => node.placed.x)
  return { min: Math.min(...xs, 0), width: Math.max(1, Math.max(...xs, 0) - Math.min(...xs, 0)) }
})

function arriveDelay(x: number): number {
  return Math.round(((x - arriveSpan.value.min) / arriveSpan.value.width) * 360)
}

/** Remounts the frames and edges after every relayout, so they fade in once nodes have moved. */
const settleKey = ref(0)
watch(
  () => props.layout,
  () => (settleKey.value += 1)
)

const worldTransform = computed(() => `translate(${camera.value.x} ${camera.value.y}) scale(${camera.value.k})`)
const dotsTransform = computed(() => `translate(${camera.value.x} ${camera.value.y}) scale(${camera.value.k})`)

let observer: ResizeObserver | null = null
onMounted(() => {
  if (!host.value) return
  const measure = (): void => {
    const box = host.value?.getBoundingClientRect()
    if (box) viewport.value = { width: box.width, height: box.height }
  }
  measure()
  if (typeof ResizeObserver !== 'undefined') {
    observer = new ResizeObserver(measure)
    observer.observe(host.value)
  }
})
onUnmounted(() => observer?.disconnect())

function pointerIn(event: { clientX: number; clientY: number }): { x: number; y: number } {
  const box = host.value?.getBoundingClientRect()
  return { x: event.clientX - (box?.left ?? 0), y: event.clientY - (box?.top ?? 0) }
}

function onWheel(event: WheelEvent): void {
  event.preventDefault()
  if (event.ctrlKey || event.metaKey) {
    set(zoomAt(camera.value, Math.exp(-event.deltaY * 0.0022), pointerIn(event)))
    return
  }
  set(panBy(camera.value, -event.deltaX, -event.deltaY))
}

let drag: { id: number; x: number; y: number; moved: number } | null = null

function onPointerDown(event: PointerEvent): void {
  if (event.button !== 0) return
  const target = event.target as Element | null
  if (target?.closest('[data-node-id]')) return
  drag = { id: event.pointerId, x: event.clientX, y: event.clientY, moved: 0 }
  host.value?.setPointerCapture?.(event.pointerId)
}

function onPointerMove(event: PointerEvent): void {
  if (!drag || drag.id !== event.pointerId) return
  const dx = event.clientX - drag.x
  const dy = event.clientY - drag.y
  drag = { ...drag, x: event.clientX, y: event.clientY, moved: drag.moved + Math.abs(dx) + Math.abs(dy) }
  set(panBy(camera.value, dx, dy))
}

function onPointerUp(event: PointerEvent): void {
  if (!drag || drag.id !== event.pointerId) return
  if (drag.moved < 4) emit('select', null)
  drag = null
}

/** Brings a node into view if it is not already, without changing the zoom. */
function reveal(id: string): void {
  const box = props.layout.nodes.get(id) ?? props.layout.clusters.get(id)
  if (!box || isInView(camera.value, box, viewport.value, 72)) return
  animateTo(centreOn(camera.value, { x: box.x + box.width / 2, y: box.y + box.height / 2 }, viewport.value))
}

defineExpose({ reveal })
</script>

<template>
  <div
    ref="host"
    class="diagram-canvas relative h-full w-full touch-none select-none overflow-hidden"
    :class="drag ? 'cursor-grabbing' : 'cursor-grab'"
    data-testid="diagram-canvas"
    @wheel="onWheel"
    @pointerdown="onPointerDown"
    @pointermove="onPointerMove"
    @pointerup="onPointerUp"
    @pointercancel="onPointerUp"
  >
    <svg class="absolute inset-0 h-full w-full" role="group" aria-label="Diagram">
      <defs>
        <pattern :id="patternId" width="24" height="24" patternUnits="userSpaceOnUse" :patternTransform="dotsTransform">
          <circle cx="12" cy="12" r="0.9" class="canvas-dot" />
        </pattern>
      </defs>
      <rect width="100%" height="100%" :fill="`url(#${patternId})`" />
      <g :key="diagramKey" :transform="worldTransform">
        <g :key="settleKey" class="canvas-settle">
          <DiagramCluster
            v-for="cluster in clusters"
            :key="cluster.item.id"
            :placed="cluster.placed"
            :item="cluster.item"
            :emphasis="nodeEmphasis(spotlight, cluster.item.id)"
            @select="emit('select', $event)"
            @toggle-fold="emit('toggleFold', $event)"
          />
          <DiagramEdge
            v-for="edge in edges"
            :key="edge.item.key"
            :placed="edge.placed"
            :item="edge.item"
            :emphasis="edgeEmphasis(spotlight, edge.item.key)"
          />
        </g>
        <DiagramNode
          v-for="node in nodes"
          :key="node.item.id"
          :placed="node.placed"
          :item="node.item"
          :emphasis="nodeEmphasis(spotlight, node.item.id)"
          :arrive-delay="arriveDelay(node.placed.x)"
          @select="emit('select', $event)"
          @toggle-fold="emit('toggleFold', $event)"
        />
      </g>
    </svg>
  </div>
</template>

<style scoped>
.canvas-dot {
  fill: color-mix(in srgb, var(--color-text-subtle) 38%, transparent);
}

/* Frames and edges come back once nodes have finished moving to a new layout, instead of
   stretching through the move. The first paint of a diagram uses the same delay. */
.canvas-settle {
  animation: canvas-settle 220ms ease 200ms both;
}

@keyframes canvas-settle {
  from {
    opacity: 0;
  }
}
</style>
