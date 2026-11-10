<script setup lang="ts">
import { computed } from 'vue'
import GraphDrawing from '@/features/diagram/components/graph/GraphDrawing.vue'
import NativeDrawing from '@/features/diagram/components/native/NativeDrawing.vue'
import SequenceDrawing from '@/features/diagram/components/sequence/SequenceDrawing.vue'
import type { SheetDrawing } from '@/features/diagram/lib/sheet-drawing'

/**
 * A drawn sheet at rest, for a page that is not the canvas: no camera, nothing selectable, no
 * links. It scales down to the width it is given and keeps its proportions.
 */
const props = defineProps<{ drawing: SheetDrawing }>()

const NO_LINKS = {}

const viewBox = computed(() => {
  const { x, y, width, height } = props.drawing.bounds
  return `${x} ${y} ${width} ${height}`
})
</script>

<template>
  <svg class="static-diagram" role="img" aria-label="Diagram" :viewBox="viewBox" :width="drawing.bounds.width" :height="drawing.bounds.height">
    <SequenceDrawing v-if="drawing.drawing === 'sequence'" :scene="drawing.scene" :layout="drawing.layout" :links="NO_LINKS" :spotlight="null" />
    <GraphDrawing v-else-if="drawing.drawing === 'graph'" :scene="drawing.scene" :layout="drawing.layout" :links="NO_LINKS" :spotlight="null" />
    <NativeDrawing v-else :svg="drawing.svg" />
  </svg>
</template>
