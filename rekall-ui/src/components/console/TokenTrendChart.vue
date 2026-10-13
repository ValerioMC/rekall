<script setup lang="ts">
import { computed, ref } from 'vue'
import { formatTokens } from '@/common/format/token-count'
import { chartGeometry, niceCeiling, shortDay, type ChartBox, type TrendPoint } from '@/common/token/token-trend'

/**
 * A task's running token total, one point per day. The readout above the plot names the day under
 * the pointer (or the arrow keys) and, with none, the latest day; the plot itself is decoration.
 */
const props = defineProps<{ points: readonly TrendPoint[]; color: string }>()

const BOX: ChartBox = { width: 340, height: 124, top: 8, right: 8, bottom: 20, left: 38 }
const GRID_FRACTIONS = [0, 0.5, 1] as const
const MIDDLE_LABEL_MIN_POINTS = 7

const hovered = ref<number | null>(null)
const plot = ref<SVGSVGElement | null>(null)

const lastIndex = computed(() => props.points.length - 1)
const activeIndex = computed(() => Math.min(hovered.value ?? lastIndex.value, lastIndex.value))
const active = computed(() => props.points[activeIndex.value] ?? null)
const ceiling = computed(() => niceCeiling(props.points[lastIndex.value]?.cumulative ?? 0))
const geometry = computed(() =>
  chartGeometry(props.points.map((point) => point.cumulative), ceiling.value, BOX)
)
const floor = BOX.height - BOX.bottom

const grid = computed(() =>
  GRID_FRACTIONS.map((fraction) => ({
    label: formatTokens(ceiling.value * fraction),
    y: floor - (floor - BOX.top) * fraction
  }))
)

const edgeLabels = computed(() => {
  const first = props.points[0]
  const last = props.points[lastIndex.value]
  if (!first || !last) return []
  const labels = [
    { text: shortDay(first.day), x: geometry.value.xs[0] ?? BOX.left, anchor: 'start' },
    { text: shortDay(last.day), x: geometry.value.xs[lastIndex.value] ?? BOX.width, anchor: 'end' }
  ]
  const middle = Math.floor(lastIndex.value / 2)
  if (props.points.length >= MIDDLE_LABEL_MIN_POINTS) {
    labels.splice(1, 0, { text: shortDay(props.points[middle]!.day), x: geometry.value.xs[middle] ?? 0, anchor: 'middle' })
  }
  return labels
})

function indexAt(clientX: number): number {
  const rect = plot.value?.getBoundingClientRect()
  if (!rect || rect.width === 0 || lastIndex.value < 1) return lastIndex.value
  const x = ((clientX - rect.left) / rect.width) * BOX.width
  const share = (x - BOX.left) / (BOX.width - BOX.left - BOX.right)
  return Math.min(lastIndex.value, Math.max(0, Math.round(share * lastIndex.value)))
}

function onKeydown(event: KeyboardEvent): void {
  const moves: Record<string, number> = {
    ArrowLeft: activeIndex.value - 1,
    ArrowRight: activeIndex.value + 1,
    Home: 0,
    End: lastIndex.value
  }
  const next = moves[event.key]
  if (next === undefined) return
  event.preventDefault()
  hovered.value = Math.min(lastIndex.value, Math.max(0, next))
}
</script>

<template>
  <div
    class="focus-ring rounded-control"
    tabindex="0"
    role="group"
    aria-label="Running total of tokens generated per day. Use the arrow keys to read a day."
    data-testid="token-trend-chart"
    @keydown="onKeydown"
    @blur="hovered = null"
  >
    <p class="flex items-baseline justify-between gap-3 text-caption" aria-live="polite" data-testid="token-trend-readout">
      <span class="text-text-muted">{{ active ? shortDay(active.day) : '' }}<span class="sr-only">, </span></span>
      <span class="font-mono tabular-nums text-text">
        {{ active ? formatTokens(active.cumulative) : '' }}
        <span v-if="active && active.generated > 0" class="text-text-subtle">+{{ formatTokens(active.generated) }}</span>
      </span>
    </p>

    <svg
      ref="plot"
      class="mt-1 block w-full touch-none select-none"
      :viewBox="`0 0 ${BOX.width} ${BOX.height}`"
      aria-hidden="true"
      @pointermove="hovered = indexAt($event.clientX)"
      @pointerleave="hovered = null"
    >
      <g v-for="line in grid" :key="line.y">
        <line :x1="BOX.left" :x2="BOX.width - BOX.right" :y1="line.y" :y2="line.y" stroke="var(--color-border)" stroke-width="1" />
        <text :x="BOX.left - 6" :y="line.y + 3" text-anchor="end" class="fill-text-subtle font-mono" font-size="9.5">{{ line.label }}</text>
      </g>
      <path :d="geometry.area" :fill="color" fill-opacity="0.14" />
      <path :d="geometry.line" fill="none" :stroke="color" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" />
      <g v-if="active">
        <line
          :x1="geometry.xs[activeIndex]"
          :x2="geometry.xs[activeIndex]"
          :y1="BOX.top"
          :y2="floor"
          stroke="var(--color-border-strong)"
          stroke-dasharray="2 3"
        />
        <circle :cx="geometry.xs[activeIndex]" :cy="geometry.ys[activeIndex]" r="3.5" :fill="color" stroke="var(--color-surface)" stroke-width="1.5" />
      </g>
      <text
        v-for="label in edgeLabels"
        :key="label.x"
        :x="label.x"
        :y="BOX.height - 5"
        :text-anchor="label.anchor"
        class="fill-text-subtle"
        font-size="9.5"
      >{{ label.text }}</text>
    </svg>
  </div>
</template>
