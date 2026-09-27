<script setup lang="ts">
import { computed, ref } from 'vue'
import { useDiagramCamera } from '@/composables/useDiagramCamera'
import { centreOn } from '@/common/diagram/viewport'
import { kindStyle } from '@/common/diagram/kind-style'
import type { DiagramLayout } from '@/common/diagram/graph-layout'
import type { Projection } from '@/common/diagram/graph-projection'

/**
 * The whole diagram in a thumbnail, with the part on screen outlined in the accent. Pressing
 * or dragging on it moves the camera there.
 */
const props = defineProps<{ layout: DiagramLayout; projection: Projection }>()

const { camera, viewport, set } = useDiagramCamera()
const WIDTH = 188
const HEIGHT = 120
const PADDING = 8

const surface = ref<SVGSVGElement | null>(null)

const scale = computed(() => {
  const bounds = props.layout.bounds
  return Math.min((WIDTH - PADDING * 2) / Math.max(1, bounds.width), (HEIGHT - PADDING * 2) / Math.max(1, bounds.height))
})

const offset = computed(() => ({
  x: (WIDTH - props.layout.bounds.width * scale.value) / 2 - props.layout.bounds.x * scale.value,
  y: (HEIGHT - props.layout.bounds.height * scale.value) / 2 - props.layout.bounds.y * scale.value
}))

const kinds = computed(() => new Map(props.projection.nodes.map((node) => [node.id, kindStyle(node.node.kind).tint])))

const frame = computed(() => {
  const k = camera.value.k
  const x = -camera.value.x / k
  const y = -camera.value.y / k
  return {
    x: x * scale.value + offset.value.x,
    y: y * scale.value + offset.value.y,
    width: (viewport.value.width / k) * scale.value,
    height: (viewport.value.height / k) * scale.value
  }
})

let dragging = false

function moveTo(event: PointerEvent): void {
  const box = surface.value?.getBoundingClientRect()
  if (!box) return
  const world = {
    x: (event.clientX - box.left - offset.value.x) / scale.value,
    y: (event.clientY - box.top - offset.value.y) / scale.value
  }
  set(centreOn(camera.value, world, viewport.value))
}

function onDown(event: PointerEvent): void {
  dragging = true
  surface.value?.setPointerCapture?.(event.pointerId)
  moveTo(event)
}

function onMove(event: PointerEvent): void {
  if (dragging) moveTo(event)
}

function onUp(): void {
  dragging = false
}
</script>

<template>
  <div class="glass overflow-hidden rounded-[12px] border border-border shadow-lift" data-testid="diagram-minimap">
    <svg
      ref="surface"
      :width="WIDTH"
      :height="HEIGHT"
      class="block cursor-crosshair touch-none"
      role="img"
      aria-label="Overview of the whole diagram"
      @pointerdown="onDown"
      @pointermove="onMove"
      @pointerup="onUp"
      @pointercancel="onUp"
    >
      <g :transform="`translate(${offset.x} ${offset.y}) scale(${scale})`">
        <rect
          v-for="cluster in layout.clusters.values()"
          :key="cluster.id"
          :x="cluster.x"
          :y="cluster.y"
          :width="cluster.width"
          :height="cluster.height"
          rx="14"
          class="mini-cluster"
        />
        <rect
          v-for="node in layout.nodes.values()"
          :key="node.id"
          :x="node.x"
          :y="node.y"
          :width="node.width"
          :height="node.height"
          rx="10"
          :style="{ fill: `color-mix(in srgb, ${kinds.get(node.id)} 55%, transparent)` }"
        />
      </g>
      <rect :x="frame.x" :y="frame.y" :width="frame.width" :height="frame.height" rx="3" class="mini-frame" />
    </svg>
  </div>
</template>

<style scoped>
.mini-cluster {
  fill: color-mix(in srgb, var(--color-text-subtle) 8%, transparent);
  stroke: var(--color-border-strong);
  stroke-width: 6;
}

.mini-frame {
  fill: color-mix(in srgb, var(--color-accent) 6%, transparent);
  stroke: var(--color-accent);
  stroke-width: 1.2;
}
</style>
