<script setup lang="ts">
import { computed, useId } from 'vue'
import { identityHue, traceAreaPath, tracePath } from '@/common/identity'

const props = withDefaults(
  defineProps<{
    id: string
    series?: readonly number[]
    size?: 'xs' | 'sm' | 'md' | 'lg'
    points?: number
  }>(),
  { size: 'sm', points: 14, series: undefined }
)

const DIMENSIONS: Record<'xs' | 'sm' | 'md' | 'lg', { width: number; height: number }> = {
  xs: { width: 24, height: 12 },
  sm: { width: 40, height: 16 },
  md: { width: 64, height: 22 },
  lg: { width: 120, height: 36 }
}

const gradientId = `trace-fill-${useId()}`
const dims = computed(() => DIMENSIONS[props.size ?? 'sm'])
const hue = computed(() => identityHue(props.id))
const strokeD = computed(() => tracePath(props.id, props.series, { ...dims.value, points: props.points }))
const areaD = computed(() => traceAreaPath(strokeD.value, dims.value.width, dims.value.height))
</script>

<template>
  <svg
    :width="dims.width"
    :height="dims.height"
    :viewBox="`0 0 ${dims.width} ${dims.height}`"
    class="shrink-0 overflow-visible"
    aria-hidden="true"
  >
    <defs>
      <linearGradient :id="gradientId" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0%" :stop-color="hue.base" stop-opacity="0.35" />
        <stop offset="100%" :stop-color="hue.base" stop-opacity="0" />
      </linearGradient>
    </defs>
    <path :d="areaD" :fill="`url(#${gradientId})`" stroke="none" />
    <path
      :d="strokeD"
      fill="none"
      :stroke="hue.base"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
</template>
