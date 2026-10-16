<script setup lang="ts">
import { computed, ref } from 'vue'
import { useDiagramCamera } from '@/features/diagram/composables/useDiagramCamera'
import { centreOn, type Box } from '@/features/diagram/lib/viewport'
import type { TintedBox } from '@/features/diagram/lib/drawing-boxes'

/**
 * The whole sheet in a thumbnail, with the part on screen outlined in the accent. Pressing or
 * dragging on it moves the camera there.
 */
const props = defineProps<{ bounds: Box; boxes: readonly TintedBox[] }>()

const { camera, viewport, set } = useDiagramCamera()
const WIDTH = 188
const HEIGHT = 120
const PADDING = 8

const surface = ref<SVGSVGElement | null>(null)

const scale = computed(() =>
  Math.min((WIDTH - PADDING * 2) / Math.max(1, props.bounds.width), (HEIGHT - PADDING * 2) / Math.max(1, props.bounds.height))
)

const offset = computed(() => ({
  x: (WIDTH - props.bounds.width * scale.value) / 2 - props.bounds.x * scale.value,
  y: (HEIGHT - props.bounds.height * scale.value) / 2 - props.bounds.y * scale.value
}))

const frame = computed(() => {
  const k = camera.value.k
  return {
    x: (-camera.value.x / k) * scale.value + offset.value.x,
    y: (-camera.value.y / k) * scale.value + offset.value.y,
    width: (viewport.value.width / k) * scale.value,
    height: (viewport.value.height / k) * scale.value
  }
})

let dragging = false

function moveTo(event: PointerEvent): void {
  const box = surface.value?.getBoundingClientRect()
  if (!box) return
  const world = { x: (event.clientX - box.left - offset.value.x) / scale.value, y: (event.clientY - box.top - offset.value.y) / scale.value }
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
</script>

<template>
  <div class="glass overflow-hidden rounded-card border border-border shadow-lift" data-testid="diagram-minimap">
    <svg
      ref="surface"
      :width="WIDTH"
      :height="HEIGHT"
      class="block cursor-crosshair touch-none"
      role="img"
      aria-label="Overview of the whole sheet"
      @pointerdown="onDown"
      @pointermove="onMove"
      @pointerup="dragging = false"
      @pointercancel="dragging = false"
    >
      <g :transform="`translate(${offset.x} ${offset.y}) scale(${scale})`">
        <rect
          v-for="box in boxes"
          :key="box.key"
          :x="box.x"
          :y="box.y"
          :width="box.width"
          :height="box.height"
          rx="10"
          :style="{ fill: `color-mix(in srgb, ${box.tint} 55%, transparent)` }"
        />
      </g>
      <rect :x="frame.x" :y="frame.y" :width="frame.width" :height="frame.height" rx="3" class="mini-frame" />
    </svg>
  </div>
</template>

<style scoped>
.mini-frame {
  fill: color-mix(in srgb, var(--color-accent) 6%, transparent);
  stroke: var(--color-accent);
  stroke-width: 1.2;
}
</style>
