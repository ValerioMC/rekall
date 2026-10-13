<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import TokenTrendPanel from '@/components/console/TokenTrendPanel.vue'
import { describeTokens, formatTokens } from '@/common/format/token-count'
import { chartGeometry, niceCeiling, trendOf, utcToday, type ChartBox } from '@/common/token/token-trend'
import { useTokenUsageStore } from '@/stores/tokenUsage.store'
import type { TaskId } from '@/model/branded'
import type { TokenTotals } from '@/model/token-usage'

/**
 * What a task or a project generated, with the last month as a sparkline. It opens the daily
 * trend of `taskIds`. The headers it sits in clip their overflow, so the panel is drawn on the body.
 */
const props = defineProps<{
  taskIds: readonly TaskId[]
  totals: TokenTotals
  color: string
  subject: 'task' | 'project'
}>()

const SPARK: ChartBox = { width: 34, height: 12, top: 1.5, right: 1.5, bottom: 1.5, left: 1.5 }
const PANEL_WIDTH = 380
const EDGE_GAP = 8

const tokenUsage = useTokenUsageStore()

const spark = computed(() => {
  const points = trendOf(tokenUsage.days, new Set(props.taskIds), '30d', utcToday())
  const values = points.map((point) => point.cumulative)
  return chartGeometry(values, niceCeiling(values[values.length - 1] ?? 0), SPARK).line
})

const open = ref(false)
const chip = ref<HTMLElement | null>(null)
const panel = ref<HTMLElement | null>(null)
const position = ref({ top: 0, left: 0 })

function show(): void {
  const rect = chip.value?.getBoundingClientRect()
  if (rect) {
    const left = Math.min(rect.left, window.innerWidth - PANEL_WIDTH - EDGE_GAP)
    position.value = { top: rect.bottom + 6, left: Math.max(EDGE_GAP, left) }
  }
  open.value = true
}

function close(): void {
  open.value = false
}

function onPointerDown(event: Event): void {
  const target = event.target as Node | null
  if (target && (chip.value?.contains(target) || panel.value?.contains(target))) return
  close()
}

function onKeydown(event: Event): void {
  if ((event as KeyboardEvent).key !== 'Escape') return
  close()
  chip.value?.focus()
}

watch(open, (isOpen) => {
  const toggle = isOpen ? 'addEventListener' : 'removeEventListener'
  document[toggle]('pointerdown', onPointerDown)
  document[toggle]('keydown', onKeydown)
  window[toggle]('resize', close)
})

watch(() => props.taskIds.join(), close)

onBeforeUnmount(close)
</script>

<template>
  <span class="inline-flex shrink-0">
    <button
      ref="chip"
      type="button"
      class="focus-ring inline-flex h-6 cursor-pointer items-center gap-1.5 rounded-key border border-border bg-surface-raised/40 px-2 font-mono text-caption tabular-nums transition-colors duration-(--motion-quick) hover:border-border-strong hover:bg-surface-raised hover:text-text"
      :class="[
        open ? 'border-border-strong bg-surface-raised text-text' : '',
        totals.generated > 0 ? 'text-text-muted' : 'text-text-subtle'
      ]"
      :title="describeTokens(totals)"
      aria-haspopup="dialog"
      :aria-expanded="open"
      data-testid="token-chip"
      @click="open ? close() : show()"
    >
      <svg class="size-2.5 shrink-0" viewBox="0 0 12 12" fill="none" aria-hidden="true" :style="{ color }">
        <path
          d="M6 1 10.5 3.5v5L6 11 1.5 8.5v-5z"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linejoin="round"
        />
      </svg>
      {{ formatTokens(totals.generated) }}
      <svg
        v-if="spark"
        :width="SPARK.width"
        :height="SPARK.height"
        :viewBox="`0 0 ${SPARK.width} ${SPARK.height}`"
        fill="none"
        aria-hidden="true"
        data-testid="token-chip-spark"
      >
        <path :d="spark" :stroke="color" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
      <span class="sr-only">tokens generated, open the daily trend</span>
    </button>

    <Teleport to="body">
      <Transition name="popover">
        <div
          v-if="open"
          ref="panel"
          class="fixed z-(--z-overlay) overflow-hidden rounded-card border border-border-strong bg-surface shadow-modal"
          :style="{ top: `${position.top}px`, left: `${position.left}px`, width: `${PANEL_WIDTH}px` }"
          role="dialog"
          :aria-label="`Tokens generated per day on this ${subject}`"
          data-testid="token-chip-panel"
        >
          <TokenTrendPanel
            :task-ids="taskIds"
            :days="tokenUsage.days"
            :totals="totals"
            :color="color"
            :subject="subject"
          />
        </div>
      </Transition>
    </Teleport>
  </span>
</template>
