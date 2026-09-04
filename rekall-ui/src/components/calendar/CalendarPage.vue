<script setup lang="ts">
import { computed, ref } from 'vue'
import AppCatalogHeader from '@/components/catalog/AppCatalogHeader.vue'
import AppButton from '@/components/ui/AppButton.vue'
import CalendarDayCell from '@/components/calendar/CalendarDayCell.vue'
import DayDetailDialog from '@/components/calendar/DayDetailDialog.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useNow } from '@/composables/useNow'
import { formatDuration } from '@/common/format/duration'
import { summarizeByDay } from '@/common/calendar/day-summary'
import { WEEKDAY_LABELS, dateKey, isSameDay, isSameMonth, monthGridDays } from '@/common/calendar/month-grid'
import type { DaySummaryRow } from '@/common/calendar/day-summary'

/**
 * A month at a time: every day a task ran on it, with the time spent — built entirely from the
 * sessions the timer already writes, the same the way `TimeLogDialog` reads them, just grouped
 * by day instead of by task.
 */
const store = useConsoleStore()
const now = useNow()

const today = new Date()
const viewYear = ref(today.getFullYear())
const viewMonth = ref(today.getMonth())

const selectedDay = ref<{ date: Date; rows: DaySummaryRow[] } | null>(null)

const monthLabel = computed(() =>
  new Date(viewYear.value, viewMonth.value, 1).toLocaleDateString(undefined, {
    month: 'long',
    year: 'numeric'
  })
)

const summaries = computed(() => summarizeByDay(store.timeEntries, store.tasks, now.value))

const cells = computed(() =>
  monthGridDays(viewYear.value, viewMonth.value).map((date) => ({
    date,
    inMonth: isSameMonth(date, viewYear.value, viewMonth.value),
    isToday: isSameDay(date, new Date(now.value)),
    rows: summaries.value.get(dateKey(date)) ?? []
  }))
)

const monthTotalSeconds = computed(() =>
  cells.value
    .filter((cell) => cell.inMonth)
    .reduce((sum, cell) => sum + cell.rows.reduce((rowSum, row) => rowSum + row.totalSeconds, 0), 0)
)

function shiftMonth(delta: number): void {
  const shifted = new Date(viewYear.value, viewMonth.value + delta, 1)
  viewYear.value = shifted.getFullYear()
  viewMonth.value = shifted.getMonth()
}

function goToday(): void {
  const current = new Date()
  viewYear.value = current.getFullYear()
  viewMonth.value = current.getMonth()
}

function openDay(cell: { date: Date; rows: DaySummaryRow[] }): void {
  selectedDay.value = { date: cell.date, rows: cell.rows }
}
</script>

<template>
  <div class="min-h-full bg-canvas">
    <AppCatalogHeader title="Calendar">
      <template #actions>
        <span class="hidden font-mono text-[11.5px] text-text-subtle sm:inline">
          {{ formatDuration(monthTotalSeconds) }} this month
        </span>
        <AppButton variant="secondary" size="sm" data-testid="calendar-today" @click="goToday">
          Today
        </AppButton>
      </template>
    </AppCatalogHeader>

    <div class="mx-auto max-w-[1240px] px-8 py-6">
      <div class="mb-4 flex items-center gap-2">
        <button
          class="focus-ring grid size-8 place-items-center rounded-[var(--radius-control)] border border-border-strong bg-surface-raised text-text-subtle transition-colors hover:border-accent hover:bg-surface-hover hover:text-text"
          aria-label="Previous month"
          data-testid="calendar-prev"
          @click="shiftMonth(-1)"
        >
          <svg class="size-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M15 6l-6 6 6 6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
        <button
          class="focus-ring grid size-8 place-items-center rounded-[var(--radius-control)] border border-border-strong bg-surface-raised text-text-subtle transition-colors hover:border-accent hover:bg-surface-hover hover:text-text"
          aria-label="Next month"
          data-testid="calendar-next"
          @click="shiftMonth(1)"
        >
          <svg class="size-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </button>
        <h2 class="ml-1 min-w-0 truncate text-[15px] font-semibold capitalize tracking-[-0.01em] text-text">
          {{ monthLabel }}
        </h2>
      </div>

      <div v-if="store.isLoading" class="skeleton h-[600px] rounded-[var(--radius-card)]" aria-hidden="true" />

      <div v-else class="overflow-hidden rounded-[var(--radius-card)] border-l border-t border-border">
        <div class="grid grid-cols-7 border-b border-border bg-surface">
          <span
            v-for="label in WEEKDAY_LABELS"
            :key="label"
            class="border-r border-border px-2 py-1.5 eyebrow"
          >
            {{ label }}
          </span>
        </div>
        <div class="grid grid-cols-7">
          <CalendarDayCell
            v-for="cell in cells"
            :key="dateKey(cell.date)"
            :date="cell.date"
            :in-month="cell.inMonth"
            :is-today="cell.isToday"
            :rows="cell.rows"
            @open="openDay(cell)"
          />
        </div>
      </div>
    </div>

    <DayDetailDialog
      v-if="selectedDay"
      :date="selectedDay.date"
      :rows="selectedDay.rows"
      @close="selectedDay = null"
    />
  </div>
</template>
