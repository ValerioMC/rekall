<script setup lang="ts">
import { APP_LOCALE } from '@/common/format/app-locale'
import { computed } from 'vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { identityHue } from '@/common/identity'
import { formatDuration } from '@/common/format/duration'
import { capitalised } from '@/features/time/lib/month-grid'
import type { ReportDay } from '@/features/time/lib/time-report'
import type { ReportPeriod } from '@/features/time/lib/period'

const props = defineProps<{
  days: readonly ReportDay[]
  period: ReportPeriod
}>()

const preferences = usePreferencesStore()
const fullDaySeconds = computed(() => preferences.workdayGoalHours * 3600)

const peak = computed(() =>
  Math.max(fullDaySeconds.value, ...props.days.map((day) => day.totalSeconds))
)

const fullDayLine = computed(() => `${(fullDaySeconds.value / peak.value) * 100}%`)

const barWidth = computed(() => (props.period === 'week' ? '46%' : '78%'))

const columns = computed(() =>
  props.days.map((day) => ({
    day,
    isWeekend: day.date.getDay() === 0 || day.date.getDay() === 6,
    weekday: capitalised(day.date.toLocaleDateString(APP_LOCALE, { weekday: 'short' })),
    dayNumber: day.date.getDate(),
    label: `${day.date.toLocaleDateString(APP_LOCALE, { weekday: 'long', day: 'numeric', month: 'long' })}: ${formatDuration(day.totalSeconds)}`,
    segments: day.byCompany.map((slice) => ({
      companyId: slice.companyId,
      height: `${(slice.seconds / peak.value) * 100}%`,
      hue: identityHue(slice.companyId).base
    }))
  }))
)
</script>

<template>
  <div class="relative" data-testid="report-ridge">
    <div class="pointer-events-none absolute inset-x-0 bottom-6 top-0" aria-hidden="true">
      <div
        class="absolute inset-x-0 border-t border-dashed border-border-strong"
        :style="{ bottom: fullDayLine }"
      >
        <span class="absolute right-0 top-0 -translate-y-1/2 rounded-key border border-border bg-surface px-1.5 font-mono text-micro leading-[15px] text-text-muted">
          {{ preferences.workdayGoalHours }}h
        </span>
      </div>
      <div class="absolute inset-x-0 bottom-0 border-t border-border" />
    </div>

    <ul
      class="relative flex h-[136px] items-end"
      :class="period === 'month' ? 'gap-[2px]' : 'gap-[3px]'"
      role="list"
    >
      <li
        v-for="column in columns"
        :key="column.day.date.toISOString()"
        class="group relative flex h-full flex-1 flex-col items-center justify-end pb-6"
        :aria-label="column.label"
        :title="column.label"
        data-testid="ridge-column"
      >
        <!-- A weekend is marked by a faint hatch, not a darker block: on an empty week a solid
             panel reads as a hole punched in the chart, and it is only a note that these days
             were not expected to fill. -->
        <span
          v-if="column.isWeekend"
          class="pointer-events-none absolute inset-x-0 bottom-6 top-0 rounded-t-key"
          style="
            background-image: repeating-linear-gradient(
              135deg,
              color-mix(in srgb, var(--color-border-strong) 55%, transparent) 0 1px,
              transparent 1px 7px
            );
          "
          aria-hidden="true"
        />

        <span
          v-if="column.day.totalSeconds === 0"
          class="relative h-px bg-border-strong"
          :style="{ width: barWidth }"
          aria-hidden="true"
        />
        <span
          v-for="(segment, index) in column.segments"
          :key="segment.companyId"
          class="relative transition-[filter] duration-(--motion-quick) group-hover:brightness-110"
          :class="index === 0 ? 'rounded-t-key' : ''"
          :style="{
            height: segment.height,
            width: barWidth,
            background: segment.hue,
            minHeight: '3px',
            boxShadow: index === 0 ? 'inset 0 1px 0 rgb(255 255 255 / 0.22)' : undefined
          }"
          aria-hidden="true"
        />

        <span
          class="pointer-events-none absolute inset-x-0 -top-1 text-center font-mono text-micro text-text opacity-0 transition-opacity group-hover:opacity-100"
          aria-hidden="true"
        >
          {{ formatDuration(column.day.totalSeconds) }}
        </span>

        <span
          class="absolute inset-x-0 bottom-0 truncate text-center text-micro leading-6"
          :class="column.day.totalSeconds > 0 ? 'text-text-muted' : 'text-text-subtle'"
          aria-hidden="true"
        >
          <template v-if="period === 'week'">{{ column.weekday }}</template>
          <template v-else>{{ column.dayNumber }}</template>
        </span>
      </li>
    </ul>
  </div>
</template>
