<script setup lang="ts">
import { computed } from 'vue'
import { formatDuration } from '@/common/format/duration'
import { identityHue } from '@/common/identity'
import type { DaySummaryRow } from '@/common/calendar/day-summary'

/**
 * One cell of the month grid: a date, and what got worked on it.
 *
 * Capped at three rows plus an overflow count rather than growing the cell to fit — a day with
 * eight tasks on it would otherwise stretch every row in the grid to match, and the calendar
 * stops reading as a calendar the moment the weeks are different heights.
 */
const props = defineProps<{
  date: Date
  inMonth: boolean
  isToday: boolean
  rows: readonly DaySummaryRow[]
}>()

defineEmits<{ open: [] }>()

const VISIBLE = 3
const HEAT_MAX_SECONDS = 4 * 3600
const HEAT_MAX_ALPHA = 0.16

const visibleRows = computed(() => props.rows.slice(0, VISIBLE))
const overflow = computed(() => Math.max(0, props.rows.length - VISIBLE))

const projectTotals = computed(() => {
  const totals = new Map<string, number>()
  for (const row of props.rows) {
    const key = row.projectId ?? row.taskId
    totals.set(key, (totals.get(key) ?? 0) + row.totalSeconds)
  }
  return [...totals.entries()].map(([id, seconds]) => ({ id, seconds, hue: identityHue(id) }))
})

const dayTotalSeconds = computed(() => props.rows.reduce((sum, row) => sum + row.totalSeconds, 0))

const dominantId = computed<string | null>(() => {
  let best: string | null = null
  let bestSeconds = -1
  for (const share of projectTotals.value) {
    if (share.seconds > bestSeconds) {
      best = share.id
      bestSeconds = share.seconds
    }
  }
  return best
})

const heatOpacity = computed(() => Math.min(1, dayTotalSeconds.value / HEAT_MAX_SECONDS) * HEAT_MAX_ALPHA)

function rowKey(row: DaySummaryRow): string {
  return row.projectId ?? row.taskId
}
</script>

<template>
  <button
    class="focus-ring relative flex min-h-[104px] flex-col items-stretch gap-1 border-b border-r border-border p-1.5 text-left transition-colors hover:bg-surface-raised"
    :class="inMonth ? 'bg-surface' : 'bg-canvas'"
    data-testid="calendar-day-cell"
    @click="$emit('open')"
  >
    <div
      v-if="dominantId"
      class="pointer-events-none absolute inset-0"
      :style="{ backgroundColor: identityHue(dominantId).base, opacity: heatOpacity }"
      aria-hidden="true"
    />

    <span class="relative flex items-center justify-end">
      <span
        class="grid size-6 place-items-center rounded-full text-[12px] tabular-nums"
        :class="[
          isToday ? 'bg-accent font-semibold text-accent-ink' : 'text-text-subtle',
          !inMonth && !isToday ? 'opacity-40' : ''
        ]"
      >
        {{ date.getDate() }}
      </span>
    </span>

    <div v-if="projectTotals.length > 1" class="relative flex h-[3px] w-full overflow-hidden rounded-full bg-border" aria-hidden="true">
      <span
        v-for="share in projectTotals"
        :key="share.id"
        :style="{ backgroundColor: share.hue.base, flexGrow: share.seconds, flexBasis: '0%' }"
      />
    </div>

    <span class="relative flex min-h-0 flex-1 flex-col gap-1">
      <span
        v-for="row in visibleRows"
        :key="row.taskId"
        class="identity-rail flex items-center gap-1 rounded-[5px] px-1.5 py-[3px]"
        :class="row.isRunning ? 'bg-accent-soft' : 'bg-surface-raised'"
        :style="{ '--identity-color': identityHue(rowKey(row)).base }"
      >
        <span
          v-if="row.isRunning"
          class="size-1 shrink-0 animate-pulse rounded-full bg-accent"
          aria-hidden="true"
        />
        <span
          class="min-w-0 flex-1 truncate text-[10.5px]"
          :class="row.isRunning ? 'text-accent' : 'text-text-muted'"
        >
          {{ row.taskTitle }}
        </span>
        <span class="shrink-0 font-mono text-[9.5px] text-text-subtle">
          {{ formatDuration(row.totalSeconds) }}
        </span>
      </span>

      <span v-if="overflow > 0" class="px-1.5 text-[10px] text-text-subtle">
        +{{ overflow }} more
      </span>
    </span>
  </button>
</template>
