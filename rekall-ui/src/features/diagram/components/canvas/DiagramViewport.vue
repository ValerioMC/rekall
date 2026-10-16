<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, useId } from 'vue'
import { useDiagramCamera } from '@/features/diagram/composables/useDiagramCamera'
import { panBy, zoomAt } from '@/features/diagram/lib/viewport'

/**
 * The drawing surface: an SVG world under the stage's camera. A plain wheel or two fingers pan,
 * a pinch or ⌘/Ctrl + wheel zooms around the pointer, dragging the background pans, and a click
 * on it clears the selection. What is drawn comes in through the slot.
 */
defineProps<{ worldKey: string }>()

const emit = defineEmits<{ clear: [] }>()

const { camera, viewport, set } = useDiagramCamera()
const host = ref<HTMLElement | null>(null)
const patternId = `diagram-dots-${useId()}`

const worldTransform = computed(() => `translate(${camera.value.x} ${camera.value.y}) scale(${camera.value.k})`)

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

const drag = ref<{ id: number; x: number; y: number; moved: number } | null>(null)

function onPointerDown(event: PointerEvent): void {
  if (event.button !== 0) return
  if ((event.target as Element | null)?.closest('[data-element]')) return
  drag.value = { id: event.pointerId, x: event.clientX, y: event.clientY, moved: 0 }
  host.value?.setPointerCapture?.(event.pointerId)
}

function onPointerMove(event: PointerEvent): void {
  const current = drag.value
  if (!current || current.id !== event.pointerId) return
  const dx = event.clientX - current.x
  const dy = event.clientY - current.y
  drag.value = { ...current, x: event.clientX, y: event.clientY, moved: current.moved + Math.abs(dx) + Math.abs(dy) }
  set(panBy(camera.value, dx, dy))
}

function onPointerUp(event: PointerEvent): void {
  const current = drag.value
  if (!current || current.id !== event.pointerId) return
  if (current.moved < 4) emit('clear')
  drag.value = null
}
</script>

<template>
  <div
    ref="host"
    class="relative h-full w-full touch-none select-none overflow-hidden"
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
        <pattern :id="patternId" width="24" height="24" patternUnits="userSpaceOnUse" :patternTransform="worldTransform">
          <circle cx="12" cy="12" r="0.9" class="canvas-dot" />
        </pattern>
      </defs>
      <rect width="100%" height="100%" :fill="`url(#${patternId})`" />
      <g :key="worldKey" :transform="worldTransform">
        <slot />
      </g>
    </svg>
  </div>
</template>

<style scoped>
.canvas-dot {
  fill: color-mix(in srgb, var(--color-text-subtle) 38%, transparent);
}
</style>
