<script setup lang="ts">
import { computed, ref } from 'vue'
import AppCatalogHeader from '@/components/catalog/AppCatalogHeader.vue'
import CalendarDayCell from '@/components/calendar/CalendarDayCell.vue'
import DayDetailDialog from '@/components/calendar/DayDetailDialog.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useNow } from '@/composables/useNow'
import { formatDuration } from '@/common/format/duration'
import { summarizeByDay } from '@/common/calendar/day-summary'
import { totalSeconds } from '@/common/calendar/day-shares'
import { weekdayLabels, dateKey, isSameDay, isSameMonth, monthGridDays } from '@/common/calendar/month-grid'
import type { DaySummaryRow } from '@/common/calendar/day-summary'

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

interface DayCell {
  readonly date: Date
  readonly inMonth: boolean
  readonly isToday: boolean
  readonly isWeekend: boolean
  readonly rows: DaySummaryRow[]
}

const cells = computed<DayCell[]>(() =>
  monthGridDays(viewYear.value, viewMonth.value).map((date) => ({
    date,
    inMonth: isSameMonth(date, viewYear.value, viewMonth.value),
    isToday: isSameDay(date, new Date(now.value)),
    isWeekend: date.getDay() === 0 || date.getDay() === 6,
    rows: summaries.value.get(dateKey(date)) ?? []
  }))
)

/** The grid's six rows, each with the time tracked across its seven days. */
const weeks = computed(() =>
  Array.from({ length: cells.value.length / 7 }, (_, index) => {
    const days = cells.value.slice(index * 7, index * 7 + 7)
    return { key: dateKey(days[0]!.date), days, seconds: days.reduce((sum, day) => sum + totalSeconds(day.rows), 0) }
  })
)

const monthTotalSeconds = computed(() =>
  cells.value.filter((cell) => cell.inMonth).reduce((sum, cell) => sum + totalSeconds(cell.rows), 0)
)

const trackedDays = computed(() => cells.value.filter((cell) => cell.inMonth && cell.rows.length).length)

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

function openDay(cell: DayCell): void {
  selectedDay.value = { date: cell.date, rows: cell.rows }
}

const WEEKDAY_LABELS = weekdayLabels()
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto bg-canvas">
    <AppCatalogHeader title="Calendar" />

    <div class="mx-auto max-w-[1320px] px-8 py-6">
      <div class="mb-5 flex flex-wrap items-end gap-x-6 gap-y-3">
        <div class="min-w-0">
          <h2 class="truncate text-heading font-semibold capitalize tracking-[-0.01em] text-text">{{ monthLabel }}</h2>
          <p class="mt-0.5 text-small text-text-subtle">
            <span class="font-mono tabular-nums text-text">{{ formatDuration(monthTotalSeconds) }}</span>
            tracked over {{ trackedDays }} day{{ trackedDays === 1 ? '' : 's' }}
          </p>
        </div>

        <div class="header-capsule ml-auto" role="group" aria-label="Month">
          <button type="button" class="header-key focus-ring" aria-label="Previous month" data-testid="calendar-prev" @click="shiftMonth(-1)">
            <svg class="size-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="M15 6l-6 6 6 6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>
          <button type="button" class="status-segment focus-ring text-text-muted hover:text-text" data-testid="calendar-today" @click="goToday">
            Today
          </button>
          <button type="button" class="header-key focus-ring" aria-label="Next month" data-testid="calendar-next" @click="shiftMonth(1)">
            <svg class="size-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
              <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>
        </div>
      </div>

      <div v-if="store.isLoading" class="skeleton h-[600px] rounded-card" aria-hidden="true" />

      <div v-else class="calendar-grid overflow-hidden rounded-card border-l border-t border-border">
        <div class="contents">
          <span v-for="label in WEEKDAY_LABELS" :key="label" class="eyebrow border-b border-r border-border bg-surface px-2 py-2">
            {{ label }}
          </span>
          <span class="eyebrow border-b border-r border-border bg-surface px-2 py-2 text-right">Week</span>
        </div>
        <template v-for="week in weeks" :key="week.key">
          <CalendarDayCell
            v-for="cell in week.days"
            :key="dateKey(cell.date)"
            :date="cell.date"
            :in-month="cell.inMonth"
            :is-today="cell.isToday"
            :is-weekend="cell.isWeekend"
            :rows="cell.rows"
            @open="openDay(cell)"
          />
          <span
            class="flex items-start justify-end border-b border-r border-border bg-canvas px-2 pt-2 font-mono text-caption tabular-nums"
            :class="week.seconds ? 'text-text-muted' : 'text-text-subtle'"
            data-testid="calendar-week-total"
          >
            {{ week.seconds ? formatDuration(week.seconds) : '–' }}
          </span>
        </template>
      </div>
    </div>

    <Transition name="dialog">
      <DayDetailDialog
        v-if="selectedDay"
        :date="selectedDay.date"
        :rows="selectedDay.rows"
        @close="selectedDay = null"
      />
    </Transition>
  </div>
</template>

<style scoped>
/* Seven days and a narrow week column that totals the row it ends. */
.calendar-grid {
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr)) 76px;
}
</style>
