<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import DiagramCanvas from '@/components/diagram/DiagramCanvas.vue'
import DiagramLegend from '@/components/diagram/DiagramLegend.vue'
import DiagramMinimap from '@/components/diagram/DiagramMinimap.vue'
import DiagramToolbar from '@/components/diagram/DiagramToolbar.vue'
import { provideDiagramCamera } from '@/composables/useDiagramCamera'
import { zoomAt } from '@/common/diagram/viewport'
import type { Spotlight } from '@/common/diagram/emphasis'
import type { Direction, DiagramLayout } from '@/common/diagram/graph-layout'
import type { Lens, Projection } from '@/common/diagram/graph-projection'
import type { Diagram, NodeKind } from '@/model/diagram'

/**
 * The centre of the diagram screen: the canvas under one camera, with the controls floating
 * over its corners on glass so the drawing runs underneath them to the edges.
 */
const props = defineProps<{
  diagram: Diagram
  projection: Projection
  layout: DiagramLayout
  spotlight: Spotlight | null
  lens: Lens
  direction: Direction
  query: string
  pinnedKind: NodeKind | null
  /** How many frames are folded: a fold or unfold reframes the drawing. */
  collapsedCount: number
  projectAnchor: string | null
  taskAnchor: string | null
}>()

const emit = defineEmits<{
  'update:lens': [lens: Lens]
  'update:direction': [direction: Direction]
  'update:query': [query: string]
  select: [id: string | null]
  toggleFold: [id: string]
  hoverKind: [kind: NodeKind | null]
  pinKind: [kind: NodeKind | null]
}>()

const shared = provideDiagramCamera()
const canvas = ref<InstanceType<typeof DiagramCanvas> | null>(null)
const toolbar = ref<InstanceType<typeof DiagramToolbar> | null>(null)

const hasCode = computed(() => props.diagram.graph.nodes.some((node) => node.kind === 'code'))
const centre = computed(() => ({ x: shared.viewport.value.width / 2, y: shared.viewport.value.height / 2 }))

function fit(animated = true): void {
  shared.fit(props.layout.bounds, animated)
}

function zoomBy(factor: number): void {
  shared.animateTo(zoomAt(shared.camera.value, factor, centre.value))
}

function resetZoom(): void {
  shared.animateTo(zoomAt(shared.camera.value, 1 / shared.camera.value.k, centre.value))
}

/** A new diagram lands framed; a new lens or direction glides to the new framing. */
watch(
  () => props.diagram.id,
  async () => {
    await nextTick()
    fit(false)
  }
)
watch(
  () => [props.lens, props.direction, props.collapsedCount],
  () => fit(true)
)
watch(
  () => shared.viewport.value.width > 0,
  (measured, before) => {
    if (measured && !before) fit(false)
  },
  { immediate: true }
)

defineExpose({
  fit,
  zoomIn: () => zoomBy(1.25),
  zoomOut: () => zoomBy(0.8),
  resetZoom,
  openSearch: () => toolbar.value?.openSearch(),
  reveal: (id: string) => canvas.value?.reveal(id)
})
</script>

<template>
  <div class="relative h-full min-w-0 flex-1">
    <DiagramCanvas
      ref="canvas"
      :diagram-key="diagram.id"
      :projection="projection"
      :layout="layout"
      :spotlight="spotlight"
      @select="emit('select', $event)"
      @toggle-fold="emit('toggleFold', $event)"
    />

    <!-- Overlays sit on glass over the canvas; each takes pointer events only where it is drawn. -->
    <div class="pointer-events-none absolute inset-x-4 top-4 flex items-start gap-3">
      <div class="glass pointer-events-auto min-w-0 max-w-[480px] rounded-[12px] border border-border px-4 py-3 shadow-lift" data-testid="diagram-title-card">
        <h2 class="truncate text-[15px] font-semibold tracking-[-0.01em] text-text">{{ diagram.title }}</h2>
        <p v-if="diagram.question" class="mt-0.5 line-clamp-2 text-[12.5px] leading-snug text-text-muted" :title="diagram.question">
          {{ diagram.question }}
        </p>
        <div class="mt-2 flex flex-wrap items-center gap-1.5 text-[11px]">
          <span v-if="projectAnchor" class="anchor-chip px-1.5 py-0.5">{{ projectAnchor }}</span>
          <span v-if="taskAnchor" class="anchor-chip px-1.5 py-0.5">{{ taskAnchor }}</span>
          <span class="font-mono tabular-nums text-text-subtle">
            {{ diagram.nodeCount }} elements · {{ diagram.edgeCount }} relations
          </span>
        </div>
      </div>
      <div class="pointer-events-auto ml-auto shrink-0">
        <DiagramToolbar
          ref="toolbar"
          :lens="lens"
          :direction="direction"
          :query="query"
          :scale="shared.camera.value.k"
          :has-code="hasCode"
          @update:lens="emit('update:lens', $event)"
          @update:direction="emit('update:direction', $event)"
          @update:query="emit('update:query', $event)"
          @zoom-in="zoomBy(1.25)"
          @zoom-out="zoomBy(0.8)"
          @reset-zoom="resetZoom"
          @fit="fit(true)"
        />
      </div>
    </div>

    <div class="pointer-events-none absolute inset-x-4 bottom-4 flex items-end justify-between gap-3">
      <div class="pointer-events-auto">
        <DiagramLegend
          :nodes="diagram.graph.nodes"
          :pinned="pinnedKind"
          @hover="emit('hoverKind', $event)"
          @pin="emit('pinKind', $event)"
        />
      </div>
      <div class="pointer-events-auto">
        <DiagramMinimap :layout="layout" :projection="projection" />
      </div>
    </div>
  </div>
</template>
