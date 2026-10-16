<script setup lang="ts">
/** The home screen's opening: today's date, one sentence on what runs and what waits, and the week's tracked time. */
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { useTerminalStore } from '@/features/claude/store/terminal.store'
import { useNow } from '@/composables/useNow'
import { situationOf } from '@/common/home/situation'
import { trackedLastDays } from '@/features/time/lib/tracked-days'
import { formatDuration } from '@/common/format/duration'

const BAR_MAX_PX = 40
const BAR_MIN_PX = 3

const { reviewQueue, timeEntries } = storeToRefs(useConsoleStore())
const { terminals } = storeToRefs(useTerminalStore())
const now = useNow(60_000)

const dateLabel = computed(() =>
  new Intl.DateTimeFormat(undefined, { weekday: 'long', day: 'numeric', month: 'long' }).format(now.value)
)
const weekdayFormat = new Intl.DateTimeFormat(undefined, { weekday: 'narrow' })
const weekdayLongFormat = new Intl.DateTimeFormat(undefined, { weekday: 'long' })

const situation = computed(() =>
  situationOf(terminals.value.filter((terminal) => terminal.live).length, reviewQueue.value.length)
)

const days = computed(() => trackedLastDays(timeEntries.value, now.value))
const today = computed(() => days.value[days.value.length - 1]?.seconds ?? 0)

const bars = computed(() => {
  const longest = Math.max(...days.value.map((day) => day.seconds), 1)
  return days.value.map((day) => ({
    ...day,
    height: `${day.seconds === 0 ? BAR_MIN_PX : Math.max(BAR_MIN_PX + 2, Math.round((day.seconds / longest) * BAR_MAX_PX))}px`,
    initial: weekdayFormat.format(day.date)
  }))
})

const weekSummary = computed(
  () =>
    'Tracked this week: ' +
    days.value
      .map((day) => `${weekdayLongFormat.format(day.date)} ${formatDuration(day.seconds)}`)
      .join(', ')
)
</script>

<template>
  <section class="flex flex-wrap items-end justify-between gap-x-10 gap-y-6" data-testid="home-situation">
    <div class="min-w-0 max-w-[34rem]">
      <p class="eyebrow uppercase tracking-[0.12em]">{{ dateLabel }}</p>
      <h1 class="mt-2 text-display font-light leading-tight tracking-[-0.02em] text-text @3xl:text-hero">
        {{ situation.working }}
        <span
          class="font-semibold"
          :class="situation.urgent ? 'text-accent-strong' : 'text-text-muted'"
          data-testid="home-situation-waiting"
        >
          {{ situation.waiting }}
        </span>
      </h1>
    </div>

    <div class="shrink-0" data-testid="home-week">
      <p class="eyebrow mb-2 uppercase tracking-[0.12em]">This week</p>
      <div class="flex items-end gap-2">
        <div class="flex items-end gap-1.5" role="img" :aria-label="weekSummary">
          <span v-for="day in bars" :key="day.key" class="flex flex-col items-center gap-1" :title="formatDuration(day.seconds)">
            <span
              class="w-2.5 rounded-key transition-[height] duration-(--motion-move)"
              :class="day.isToday ? 'bg-accent' : 'bg-border-strong'"
              :style="{ height: day.height }"
            />
            <span class="font-mono text-micro text-text-subtle" aria-hidden="true">{{ day.initial }}</span>
          </span>
        </div>
        <p class="ml-3 pb-5 font-mono text-heading tabular-nums text-text" data-testid="home-today">
          {{ formatDuration(today) }}
          <span class="font-sans text-caption text-text-subtle">today</span>
        </p>
      </div>
    </div>
  </section>
</template>
