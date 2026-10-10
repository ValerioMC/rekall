<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import DiagramViewport from '@/components/diagram/canvas/DiagramViewport.vue'
import GraphDrawing from '@/components/diagram/graph/GraphDrawing.vue'
import NativeDrawing from '@/components/diagram/native/NativeDrawing.vue'
import SequenceDrawing from '@/components/diagram/sequence/SequenceDrawing.vue'
import DiagramMinimap from './DiagramMinimap.vue'
import DiagramTitleCard from './DiagramTitleCard.vue'
import DiagramToolbar from './DiagramToolbar.vue'
import ElementPanel from './ElementPanel.vue'
import SheetTabs from './SheetTabs.vue'
import { provideDiagramCamera } from '@/composables/useDiagramCamera'
import { useModalGate } from '@/composables/useModalGate'
import { useSheetDrawing } from '@/composables/useSheetDrawing'
import { connectionsOf, graphElement, sequenceElement } from '@/common/diagram/element-detail'
import { graphSpotlight, sequenceSpotlight, callsFor } from '@/common/diagram/spotlight'
import { minimapBoxesOf, elementBoxOf } from '@/common/diagram/drawing-boxes'
import { centreOn, isInView, zoomAt } from '@/common/diagram/viewport'
import { centre } from '@/common/diagram/layout/geometry'
import type { Diagram, DiagramSummary } from '@/model/diagram'
import type { DiagramId } from '@/model/branded'

/**
 * One diagram, full bleed: the open sheet on the canvas, the controls floating over its edges,
 * and the picked element's code and calls in a panel that appears only while something is picked.
 */
const props = defineProps<{ diagram: Diagram; siblings: readonly DiagramSummary[]; projectAnchor: string | null; taskAnchor: string | null }>()

const emit = defineEmits<{ back: []; open: [id: DiagramId]; remove: [] }>()

/** Room the element panel takes on the right, kept clear when the canvas reveals an element. */
const PANEL_ROOM = 432

const shared = provideDiagramCamera()
const { isModalOpen } = useModalGate()
const toolbar = ref<InstanceType<typeof DiagramToolbar> | null>(null)
const sheetIndex = ref(0)
const selected = ref<string | null>(null)
const query = ref('')

const sheet = computed(() => props.diagram.sheets[sheetIndex.value] ?? null)
const { drawing, failure, loading } = useSheetDrawing(sheet)
const links = computed(() => sheet.value?.links ?? {})

const spotlight = computed(() => {
  const drawn = drawing.value
  if (drawn?.drawing === 'sequence') return sequenceSpotlight(drawn.scene, drawn.calls, selected.value, query.value)
  if (drawn?.drawing === 'graph') return graphSpotlight(drawn.scene, selected.value, query.value)
  return null
})

const detail = computed(() => {
  const drawn = drawing.value
  if (!selected.value || !drawn) return null
  if (drawn.drawing === 'sequence') return sequenceElement(drawn.scene, selected.value)
  if (drawn.drawing === 'graph') return graphElement(drawn.scene, selected.value)
  return null
})

const calls = computed(() => (drawing.value?.drawing === 'sequence' && selected.value ? callsFor(drawing.value.calls, selected.value) : null))
const connections = computed(() => (drawing.value?.drawing === 'graph' && selected.value ? connectionsOf(drawing.value.scene, selected.value) : null))
const labels = computed(
  () => new Map(drawing.value?.drawing === 'sequence' ? drawing.value.scene.participants.map((participant) => [participant.id, participant.label] as const) : [])
)
const minimap = computed(() => (drawing.value ? minimapBoxesOf(drawing.value) : []))

watch(
  () => props.diagram.id,
  () => {
    sheetIndex.value = 0
    clearSelection()
  }
)
watch(sheetIndex, clearSelection)

function clearSelection(): void {
  selected.value = null
  query.value = ''
}

function fit(animated: boolean): void {
  if (drawing.value) shared.fit(drawing.value.bounds, animated)
}

watch(drawing, async (drawn) => {
  if (!drawn) return
  await nextTick()
  fit(false)
})
watch(
  () => shared.viewport.value.width > 0,
  (measured, before) => {
    if (measured && !before) fit(false)
  },
  { immediate: true }
)

function zoomBy(factor: number): void {
  const viewport = shared.viewport.value
  shared.animateTo(zoomAt(shared.camera.value, factor, { x: viewport.width / 2, y: viewport.height / 2 }))
}

function resetZoom(): void {
  zoomBy(1 / shared.camera.value.k)
}

/** Picks an element and brings it into the part of the canvas the panel leaves clear. */
function select(key: string): void {
  selected.value = key
  const box = drawing.value ? elementBoxOf(drawing.value, key) : null
  if (!box) return
  const viewport = shared.viewport.value
  const clear = { width: Math.max(240, viewport.width - PANEL_ROOM), height: viewport.height }
  if (isInView(shared.camera.value, box, clear, 72)) return
  shared.animateTo(centreOn(shared.camera.value, centre(box), clear))
}

function onKeydown(event: KeyboardEvent): void {
  if (isModalOpen.value > 0) return
  const target = event.target as HTMLElement | null
  if (target && (/^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName) || target.isContentEditable)) return
  if (event.metaKey || event.ctrlKey || event.altKey) return
  const sheets = props.diagram.sheets.length
  const actions: Record<string, () => void> = {
    f: () => fit(true),
    '+': () => zoomBy(1.25),
    '=': () => zoomBy(1.25),
    '-': () => zoomBy(0.8),
    '0': resetZoom,
    '/': () => toolbar.value?.openSearch(),
    '[': () => (sheetIndex.value = (sheetIndex.value - 1 + sheets) % sheets),
    ']': () => (sheetIndex.value = (sheetIndex.value + 1) % sheets),
    Backspace: () => emit('back'),
    Escape: clearSelection
  }
  const action = actions[event.key]
  if (!action) return
  event.preventDefault()
  action()
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onUnmounted(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="relative h-full min-w-0 flex-1" data-testid="diagram-viewer">
    <div id="diagram-sheet" role="tabpanel" :aria-labelledby="`sheet-tab-${sheetIndex}`" class="absolute inset-0">
      <DiagramViewport :world-key="`${diagram.id}:${sheetIndex}`" @clear="selected = null">
        <SequenceDrawing v-if="drawing?.drawing === 'sequence'" :scene="drawing.scene" :layout="drawing.layout" :links="links" :spotlight="spotlight" @select="select" />
        <GraphDrawing v-else-if="drawing?.drawing === 'graph'" :scene="drawing.scene" :layout="drawing.layout" :links="links" :spotlight="spotlight" @select="select" />
        <NativeDrawing v-else-if="drawing?.drawing === 'native'" :svg="drawing.svg" />
      </DiagramViewport>

      <div v-if="loading" class="pointer-events-none absolute inset-0 grid place-items-center" role="status" aria-busy="true">
        <span class="sr-only">Drawing the sheet</span>
        <span class="skeleton h-40 w-[min(520px,70%)] rounded-card" />
      </div>
      <div v-else-if="failure" class="absolute inset-0 grid place-items-center p-8" role="alert">
        <div class="glass max-w-[560px] rounded-card border border-border p-5 shadow-lift">
          <p class="text-body font-medium text-text">Mermaid could not read this sheet.</p>
          <p class="mt-1 text-small text-text-muted">{{ failure }}</p>
          <pre class="mt-3 max-h-[240px] overflow-auto rounded-control border border-border bg-canvas p-3 font-mono text-caption text-text-muted">{{ sheet?.mermaid }}</pre>
        </div>
      </div>
    </div>

    <!-- Overlays sit on glass over the canvas; each takes pointer events only where it is drawn. -->
    <div class="pointer-events-none absolute inset-x-4 top-4 flex flex-wrap items-start gap-3">
      <DiagramTitleCard
        :diagram="diagram"
        :siblings="siblings"
        :project-anchor="projectAnchor"
        :task-anchor="taskAnchor"
        @back="emit('back')"
        @open="emit('open', $event)"
        @remove="emit('remove')"
      />
      <div v-if="diagram.sheets.length > 1" class="pointer-events-auto">
        <SheetTabs :sheets="diagram.sheets" :active="sheetIndex" @select="sheetIndex = $event" />
      </div>
      <div class="pointer-events-auto ml-auto">
        <DiagramToolbar
          ref="toolbar"
          v-model:query="query"
          :scale="shared.camera.value.k"
          :searchable="drawing?.drawing === 'sequence' || drawing?.drawing === 'graph'"
          @zoom-in="zoomBy(1.25)"
          @zoom-out="zoomBy(0.8)"
          @reset-zoom="resetZoom"
          @fit="fit(true)"
        />
      </div>
    </div>

    <div class="pointer-events-none absolute bottom-4 right-4 top-[76px] flex flex-col items-end justify-end gap-3">
      <Transition name="fade-quick">
        <ElementPanel
          v-if="detail"
          :key="detail.key"
          class="min-h-0 flex-1"
          :diagram-id="diagram.id"
          :detail="detail"
          :sources="links[detail.linkKey] ?? []"
          :calls="calls"
          :connections="connections"
          :labels="labels"
          :links="links"
          @close="selected = null"
          @select="select"
        />
      </Transition>
    </div>
    <div v-if="!detail && drawing && drawing.drawing !== 'native'" class="pointer-events-auto absolute bottom-4 right-4">
      <DiagramMinimap :bounds="drawing.bounds" :boxes="minimap" />
    </div>
  </div>
</template>
