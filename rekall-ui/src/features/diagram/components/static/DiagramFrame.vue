<script setup lang="ts">
import { ref } from 'vue'
import DiagramZoomViewer from '@/features/diagram/components/static/DiagramZoomViewer.vue'
import StaticDiagram from '@/features/diagram/components/static/StaticDiagram.vue'
import type { SheetDrawing } from '@/features/diagram/lib/sheet-drawing'

/** A diagram of a read view that can be enlarged: a key in its corner, or a click on it, opens it over the window. */
defineProps<{ drawing: SheetDrawing }>()

const enlarged = ref(false)
</script>

<template>
  <button type="button" class="rk-mermaid-expand" aria-label="Enlarge the diagram" title="Enlarge" data-testid="diagram-expand" @click="enlarged = true">
    <svg viewBox="0 0 16 16" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="M9.5 2.5h4v4M6.5 13.5h-4v-4M13.5 2.5 9 7M2.5 13.5 7 9" />
    </svg>
  </button>
  <StaticDiagram class="rk-mermaid-zoomable" :drawing="drawing" @click="enlarged = true" />
  <DiagramZoomViewer v-if="enlarged" :drawing="drawing" @close="enlarged = false" />
</template>
