<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import DiagramViewport from '@/features/diagram/components/canvas/DiagramViewport.vue'
import GraphDrawing from '@/features/diagram/components/graph/GraphDrawing.vue'
import NativeDrawing from '@/features/diagram/components/native/NativeDrawing.vue'
import SequenceDrawing from '@/features/diagram/components/sequence/SequenceDrawing.vue'
import WalkthroughBar from '@/features/diagram/components/walkthrough/WalkthroughBar.vue'
import WalkthroughCard from '@/features/diagram/components/walkthrough/WalkthroughCard.vue'
import WalkthroughPulse from '@/features/diagram/components/walkthrough/WalkthroughPulse.vue'
import DiagramMinimap from './DiagramMinimap.vue'
import DiagramTitleCard from './DiagramTitleCard.vue'
import DiagramToolbar from './DiagramToolbar.vue'
import ElementPanel from './ElementPanel.vue'
import SheetTabs from './SheetTabs.vue'
import ViewModeSwitch from './ViewModeSwitch.vue'
import { provideDiagramCamera } from '@/features/diagram/composables/useDiagramCamera'
import { useModalGate } from '@/composables/useModalGate'
import { useSheetDrawing } from '@/features/diagram/composables/useSheetDrawing'
import { useWalkthrough, type ViewMode } from '@/features/diagram/composables/useWalkthrough'
import { firstBeatInto } from '@/features/diagram/lib/walkthrough/walkthrough-beats'
import { KIND_INFO } from '@/features/diagram/lib/kinds'
import { connectionsOf, graphElement, sequenceElement } from '@/features/diagram/lib/element-detail'
import { graphSpotlight, sequenceSpotlight, callsFor } from '@/features/diagram/lib/spotlight'
import { minimapBoxesOf, elementBoxOf } from '@/features/diagram/lib/drawing-boxes'
import { centreOn, isInView, zoomAt } from '@/features/diagram/lib/viewport'
import { centre } from '@/features/diagram/lib/layout/geometry'
import type { Diagram, DiagramSummary } from '@/features/diagram/model/diagram'
import type { DiagramId } from '@/model/branded'

/**
 * One diagram, full bleed: the open sheet on the canvas, the controls floating over its edges,
 * and the picked element's code and calls in a panel that appears only while something is picked.
 * A sequence can also play as a walkthrough, its current step's card in place of that panel.
 */
const props = defineProps<{ diagram: Diagram; siblings: readonly DiagramSummary[]; projectAnchor: string | null; taskAnchor: string | null }>()

const emit = defineEmits<{ back: []; open: [id: DiagramId]; remove: [] }>()

/** Room the element panel takes on the right, kept clear when the canvas reveals an element. */
const PANEL_ROOM = 432
/** Room the walkthrough's bar takes at the foot of the canvas. */
const BAR_ROOM = 104

const shared = provideDiagramCamera()
const { isModalOpen } = useModalGate()
const toolbar = ref<InstanceType<typeof DiagramToolbar> | null>(null)
const sheetIndex = ref(0)
const selected = ref<string | null>(null)
const query = ref('')

const sheet = computed(() => props.diagram.sheets[sheetIndex.value] ?? null)
const { drawing, failure, loading } = useSheetDrawing(sheet)
const links = computed(() => sheet.value?.links ?? {})

const sheets = computed(() => props.diagram.sheets)
const walk = useWalkthrough(sheets, sheetIndex, drawing)
const mode = ref<ViewMode>(props.diagram.sheets[0]?.walkthrough ? 'walkthrough' : 'diagram')
const playable = computed(() => sheet.value?.kind === 'sequence')
const walking = computed(() => mode.value === 'walkthrough' && playable.value)
const trail = computed(() => walk.stack.value.map((frame) => frame.via ?? sheetLabel(frame.sheetIndex)))
const opensTitle = computed(() => {
  const target = walk.beat.value?.opens
  return target === null || target === undefined ? null : sheetLabel(target)
})
const placedBeat = computed(() => {
  const drawn = drawing.value
  const current = walk.beat.value
  return drawn?.drawing === 'sequence' && current ? drawn.layout.messages.get(current.message.number) ?? null : null
})

function sheetLabel(index: number): string {
  const named = props.diagram.sheets[index]
  return named ? named.title ?? KIND_INFO[named.kind].label : `Sheet ${index + 1}`
}

function changeMode(next: ViewMode): void {
  walk.playing.value = false
  clearSelection()
  mode.value = next
}

const spotlight = computed(() => {
  const drawn = drawing.value
  if (walking.value) return walk.spotlight.value
  if (drawn?.drawing === 'sequence') return sequenceSpotlight(drawn.scene, drawn.calls, selected.value, query.value)
  if (drawn?.drawing === 'graph') return graphSpotlight(drawn.scene, selected.value, query.value)
  return null
})

const detail = computed(() => {
  const drawn = drawing.value
  if (!selected.value || !drawn) return null
  if (drawn.drawing === 'sequence') return sequenceElement(drawn.scene, selected.value, walk.external.value)
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

/** Picks an element and brings it into view; while walking, a pick moves the walkthrough there instead. */
function select(key: string): void {
  if (walking.value) return walkTo(key)
  selected.value = key
  reveal(key)
}

function walkTo(key: string): void {
  const beats = walk.beats.value
  walk.goTo(key.startsWith('#') ? beats.findIndex((beat) => beat.key === key) : firstBeatInto(beats, key))
}

watch(
  () => (walking.value ? walk.beat.value?.key : null),
  async (key) => {
    if (!key) return
    await nextTick()
    reveal(key)
  }
)

/** Brings an element into the part of the canvas the panel leaves clear. */
function reveal(key: string): void {
  const box = drawing.value ? elementBoxOf(drawing.value, key) : null
  if (!box) return
  const viewport = shared.viewport.value
  const clear = { width: Math.max(240, viewport.width - PANEL_ROOM), height: Math.max(200, viewport.height - (walking.value ? BAR_ROOM : 0)) }
  if (isInView(shared.camera.value, box, clear, 72)) return
  shared.animateTo(centreOn(shared.camera.value, centre(box), clear))
}

/** Keys the walkthrough takes over while it plays; a focused button keeps Space and Enter for itself. */
function walkthroughAction(event: KeyboardEvent): (() => void) | null {
  const onButton = (event.target as HTMLElement | null)?.closest('button, [role="button"]') !== null
  if (onButton && (event.key === ' ' || event.key === 'Enter')) return null
  const nested = walk.stack.value.length > 1
  const actions: Record<string, () => void> = {
    ArrowRight: walk.next,
    ArrowLeft: walk.previous,
    ' ': walk.toggle,
    Enter: walk.stepInto,
    Home: walk.restart,
    End: () => walk.goTo(walk.beats.value.length - 1),
    Escape: nested ? walk.stepOut : () => (walk.playing.value = false)
  }
  return actions[event.key] ?? null
}

function onKeydown(event: KeyboardEvent): void {
  if (isModalOpen.value > 0) return
  const target = event.target as HTMLElement | null
  if (target && (/^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName) || target.isContentEditable)) return
  if (event.metaKey || event.ctrlKey || event.altKey) return
  const walked = walking.value ? walkthroughAction(event) : null
  if (walked) {
    event.preventDefault()
    walked()
    return
  }
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
        <template v-if="drawing?.drawing === 'sequence'">
          <SequenceDrawing :scene="drawing.scene" :layout="drawing.layout" :links="links" :spotlight="spotlight" :external="walk.external.value" @select="select" />
          <WalkthroughPulse v-if="walking && placedBeat && walk.beat.value" :key="`${sheetIndex}:${walk.beat.value.key}`" :placed="placedBeat" :motion="walk.beat.value.motion" />
        </template>
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
      <div v-if="diagram.sheets.some((candidate) => candidate.kind === 'sequence')" class="pointer-events-auto">
        <ViewModeSwitch :mode="mode" :playable="playable" @change="changeMode" />
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
      <Transition name="pane" mode="out-in">
        <WalkthroughCard
          v-if="walking && walk.beat.value"
          :key="`${sheetIndex}:${walk.beat.value.key}`"
          class="min-h-0 flex-1"
          :diagram-id="diagram.id"
          :beat="walk.beat.value"
          :position="walk.position.value"
          :count="walk.beats.value.length"
          :labels="labels"
          :external="walk.external.value"
          :sources="links[walk.beat.value.key] ?? []"
          :opens-title="opensTitle"
          @step-into="walk.stepInto"
        />
        <ElementPanel
          v-else-if="detail"
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
    <div v-if="walking" class="pointer-events-none absolute bottom-4 left-4 right-[452px] flex justify-center max-[900px]:right-4">
      <WalkthroughBar
        :beats="walk.beats.value"
        :position="walk.position.value"
        :playing="walk.playing.value"
        :pace="walk.pace.value"
        :trail="trail"
        @previous="walk.previous"
        @next="walk.next"
        @toggle="walk.toggle"
        @restart="walk.restart"
        @go-to="walk.goTo"
        @pace="walk.pace.value = $event"
        @back-to="walk.backTo"
      />
    </div>
    <div v-else-if="!detail && drawing && drawing.drawing !== 'native'" class="pointer-events-auto absolute bottom-4 right-4">
      <DiagramMinimap :bounds="drawing.bounds" :boxes="minimap" />
    </div>
  </div>
</template>
