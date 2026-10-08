<script setup lang="ts">
import { computed } from 'vue'
import { formatDuration } from '@/common/format/duration'
import { identityHue } from '@/common/identity'
import { dayLoad, projectShares, totalSeconds } from '@/common/calendar/day-shares'
import type { DaySummaryRow } from '@/common/calendar/day-summary'

/**
 * One day of the month grid. The head carries the day's total; under it, a load bar whose length
 * is the day against a working day and whose segments are its projects, so a busy day and what
 * filled it read before any title does.
 */
const props = defineProps<{
  date: Date
  inMonth: boolean
  isToday: boolean
  isWeekend: boolean
  rows: readonly DaySummaryRow[]
}>()

defineEmits<{ open: [] }>()

const VISIBLE = 3

const visibleRows = computed(() => props.rows.slice(0, VISIBLE))
const overflow = computed(() => Math.max(0, props.rows.length - VISIBLE))
const shares = computed(() => projectShares(props.rows))
const dayTotal = computed(() => totalSeconds(props.rows))
const load = computed(() => dayLoad(dayTotal.value))

const label = computed(() => {
  const day = props.date.toLocaleDateString(undefined, { weekday: 'long', day: 'numeric', month: 'long' })
  return dayTotal.value ? `${day}, ${formatDuration(dayTotal.value)} tracked` : `${day}, nothing tracked`
})

function rowHue(row: DaySummaryRow): string {
  return identityHue(row.projectId ?? row.taskId).base
}
</script>

<template>
  <button
    class="calendar-day focus-ring relative flex min-h-[112px] flex-col items-stretch gap-1.5 border-b border-r border-border px-2 pb-2 pt-1.5 text-left transition-colors duration-(--motion-quick) hover:bg-surface-hover"
    :class="[inMonth ? (isWeekend ? 'bg-surface/60' : 'bg-surface') : 'bg-canvas', isToday && 'calendar-today']"
    :aria-label="label"
    :aria-current="isToday ? 'date' : undefined"
    data-testid="calendar-day-cell"
    @click="$emit('open')"
  >
    <span class="flex h-6 items-center gap-2">
      <span
        class="grid h-6 min-w-6 place-items-center rounded-control px-1 text-small font-medium tabular-nums"
        :class="isToday ? 'bg-accent text-accent-ink' : inMonth ? 'text-text-muted' : 'text-text-subtle'"
      >
        {{ date.getDate() }}
      </span>
      <span v-if="dayTotal" class="ml-auto font-mono text-caption tabular-nums text-text" :class="!inMonth && 'text-text-subtle'">
        {{ formatDuration(dayTotal) }}
      </span>
    </span>

    <span v-if="dayTotal" class="flex h-1 w-full overflow-hidden rounded-full bg-border/60" aria-hidden="true">
      <span class="flex h-full overflow-hidden rounded-full" :style="{ width: `${Math.max(load * 100, 6)}%` }">
        <span
          v-for="share in shares"
          :key="share.id"
          class="h-full"
          :style="{ backgroundColor: identityHue(share.id).base, flexGrow: share.seconds, flexBasis: '0%' }"
        />
      </span>
    </span>

    <span class="flex min-h-0 flex-1 flex-col gap-0.5" :class="!inMonth && 'opacity-60'">
      <span
        v-for="row in visibleRows"
        :key="row.taskId"
        class="flex items-center gap-1.5 rounded-key px-1 py-px"
        :class="row.isRunning && 'bg-accent-soft'"
      >
        <span
          class="size-1.5 shrink-0 rounded-full"
          :class="row.isRunning && 'animate-pulse'"
          :style="{ backgroundColor: row.isRunning ? 'var(--color-accent)' : rowHue(row) }"
          aria-hidden="true"
        />
        <span class="min-w-0 flex-1 truncate text-caption" :class="row.isRunning ? 'text-accent' : 'text-text-muted'">
          {{ row.taskTitle }}
        </span>
        <span class="shrink-0 font-mono text-micro tabular-nums text-text-subtle max-[1279px]:hidden">
          {{ formatDuration(row.totalSeconds) }}
        </span>
      </span>

      <span v-if="overflow > 0" class="mt-auto px-1 text-micro text-text-subtle">+{{ overflow }} more</span>
    </span>
  </button>
</template>

<style scoped>
/* Today is held by a ring inside its own borders, so the grid's lines never shift around it. */
.calendar-today {
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--color-accent) 55%, transparent);
}
</style>
