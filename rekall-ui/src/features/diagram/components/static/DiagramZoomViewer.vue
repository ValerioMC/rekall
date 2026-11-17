<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import DiagramViewport from '@/features/diagram/components/canvas/DiagramViewport.vue'
import StaticDiagram from '@/features/diagram/components/static/StaticDiagram.vue'
import { provideDiagramCamera } from '@/features/diagram/composables/useDiagramCamera'
import { actionOfKey, cameraAfter, ZOOM_STEP, type ViewerAction } from '@/features/diagram/lib/viewer-keys'
import { fitCamera } from '@/features/diagram/lib/viewport'
import type { SheetDrawing } from '@/features/diagram/lib/sheet-drawing'

/**
 * A diagram of a read view enlarged over the whole window, fitted to it first. It moves and
 * zooms as the Diagrams screen does, and also from the keyboard: + and − zoom, the arrows move,
 * 0 fits, Esc closes.
 */
const props = defineProps<{ drawing: SheetDrawing }>()

const emit = defineEmits<{ close: [] }>()

const FIT_PADDING_PX = 48
const KEY =
  'focus-ring grid size-8 cursor-pointer place-items-center rounded-card text-text-muted transition-colors hover:bg-surface-raised hover:text-text'

const { camera, viewport, set, animateTo } = provideDiagramCamera()
const root = ref<HTMLElement | null>(null)
const opener = document.activeElement as HTMLElement | null
const world = computed(() => ({ x: 0, y: 0, width: props.drawing.bounds.width, height: props.drawing.bounds.height }))
const zoomPercent = computed(() => Math.round(camera.value.k * 100))

function fit(animated: boolean): void {
  const target = fitCamera(world.value, viewport.value, FIT_PADDING_PX)
  if (animated) animateTo(target)
  else set(target)
}

function perform(action: ViewerAction): void {
  if (action.kind === 'close') emit('close')
  else if (action.kind === 'fit') fit(true)
  else if (action.kind === 'zoom') animateTo(cameraAfter(action, camera.value, viewport.value))
  else set(cameraAfter(action, camera.value, viewport.value))
}

function onKeydown(event: KeyboardEvent): void {
  if (event.metaKey || event.ctrlKey || event.altKey) return
  const action = actionOfKey(event.key)
  if (!action) return
  event.preventDefault()
  event.stopPropagation()
  perform(action)
}

const stopWaiting = watch(
  () => viewport.value.width,
  (width) => {
    if (width === 0) return
    fit(false)
    stopWaiting()
  }
)

onMounted(() => root.value?.focus())
onBeforeUnmount(() => opener?.focus?.())
</script>

<template>
  <Teleport to="body">
    <div
      ref="root"
      class="fixed inset-0 z-[80] bg-canvas outline-none"
      role="dialog"
      aria-modal="true"
      aria-label="Diagram, enlarged"
      tabindex="-1"
      data-testid="diagram-zoom"
      @keydown="onKeydown"
    >
      <DiagramViewport world-key="enlarged">
        <StaticDiagram :drawing="drawing" style="overflow: visible" />
      </DiagramViewport>

      <button :class="[KEY, 'absolute right-4 top-4 border border-border bg-surface']" aria-label="Close" title="Close (Esc)" data-testid="diagram-zoom-close" @click="emit('close')">
        <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true"><path d="m4 4 8 8M12 4 4 12" /></svg>
      </button>

      <div class="absolute bottom-6 left-1/2 flex -translate-x-1/2 items-center gap-0.5 rounded-card border border-border bg-surface p-1 shadow-[var(--shadow-raised)]" role="group" aria-label="Zoom">
        <button :class="KEY" title="Zoom out (−)" aria-label="Zoom out" data-testid="diagram-zoom-out" @click="perform({ kind: 'zoom', factor: 1 / ZOOM_STEP })">
          <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true"><path d="M4 8h8" /></svg>
        </button>
        <span class="min-w-[3.25rem] px-1.5 text-center font-mono text-caption tabular-nums text-text-muted" data-testid="diagram-zoom-readout">{{ zoomPercent }}%</span>
        <button :class="KEY" title="Zoom in (+)" aria-label="Zoom in" data-testid="diagram-zoom-in" @click="perform({ kind: 'zoom', factor: ZOOM_STEP })">
          <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true"><path d="M4 8h8M8 4v8" /></svg>
        </button>
        <button :class="KEY" title="Fit to the window (0)" aria-label="Fit to the window" data-testid="diagram-zoom-fit" @click="perform({ kind: 'fit' })">
          <svg viewBox="0 0 16 16" class="size-4" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M2.5 5.5v-3h3M10.5 2.5h3v3M13.5 10.5v3h-3M5.5 13.5h-3v-3" />
          </svg>
        </button>
      </div>
    </div>
  </Teleport>
</template>
