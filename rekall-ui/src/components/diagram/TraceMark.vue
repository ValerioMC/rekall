<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import type { Provenance } from '@/model/diagram'

/**
 * The trace mark: how far to trust an element, as one small object instead of three badges.
 *
 *   outer ring  how it was arrived at: solid observed in code, dashed inferred, dotted from
 *               docs, doubled when a person stated it; absent when nobody said
 *   inner arc   the confidence, swept over a track; no arc when none was given
 *   core        filled when the element points at real code, hollow when it is only a concept
 *
 * The arc sweeps in once when the mark first appears and never again while it stays, so a
 * diagram lands with its confidences drawing themselves and then holds still.
 */
const props = withDefaults(
  defineProps<{
    confidence: number | null
    provenance: Provenance | null
    traced: boolean
    tint: string
    size?: number
    x?: number
    y?: number
  }>(),
  { size: 20, x: 0, y: 0 }
)

const ARC_RADIUS = 5.4
const CIRCUMFERENCE = 2 * Math.PI * ARC_RADIUS

const drawn = ref(false)
onMounted(() => requestAnimationFrame(() => (drawn.value = true)))

const arcLength = computed(() => CIRCUMFERENCE * Math.min(1, Math.max(0, props.confidence ?? 0)))

const ringDash = computed(() => {
  if (props.provenance === 'inferred') return '2.4 1.9'
  if (props.provenance === 'documented') return '0.01 2.3'
  return undefined
})
</script>

<template>
  <svg :x="x" :y="y" :width="size" :height="size" viewBox="0 0 20 20" fill="none" aria-hidden="true">
    <circle
      v-if="provenance !== null"
      cx="10"
      cy="10"
      r="8.9"
      :stroke="tint"
      stroke-opacity="0.6"
      stroke-width="1.1"
      stroke-linecap="round"
      :stroke-dasharray="ringDash"
    />
    <circle v-if="provenance === 'stated'" cx="10" cy="10" r="7.4" :stroke="tint" stroke-opacity="0.45" stroke-width="0.7" />
    <circle cx="10" cy="10" :r="ARC_RADIUS" stroke="var(--color-border-strong)" stroke-width="1.8" />
    <circle
      v-if="confidence !== null"
      cx="10"
      cy="10"
      :r="ARC_RADIUS"
      :stroke="tint"
      stroke-width="1.8"
      stroke-linecap="round"
      transform="rotate(-90 10 10)"
      class="trace-arc"
      :stroke-dasharray="`${CIRCUMFERENCE} ${CIRCUMFERENCE}`"
      :stroke-dashoffset="drawn ? CIRCUMFERENCE - arcLength : CIRCUMFERENCE"
    />
    <circle v-if="traced" cx="10" cy="10" r="2.1" :fill="tint" />
    <circle v-else cx="10" cy="10" r="1.8" :stroke="tint" stroke-width="1" />
  </svg>
</template>

<style scoped>
.trace-arc {
  transition: stroke-dashoffset 560ms cubic-bezier(0.16, 1, 0.3, 1);
}
</style>
