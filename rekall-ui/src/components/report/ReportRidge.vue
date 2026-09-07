<script setup lang="ts">
import { computed } from 'vue'
import { identityHue } from '@/common/identity'
import { formatDuration } from '@/common/format/duration'
import type { ReportDay } from '@/common/report/time-report'
import type { ReportPeriod } from '@/common/report/period'

const props = defineProps<{
  days: readonly ReportDay[]
  period: ReportPeriod
}>()

const FULL_DAY_SECONDS = 8 * 3600

const peak = computed(() =>
  Math.max(FULL_DAY_SECONDS, ...props.days.map((day) => day.totalSeconds))
)

const fullDayLine = computed(() => `${(FULL_DAY_SECONDS / peak.value) * 100}%`)

const barWidth = computed(() => (props.period === 'week' ? '46%' : '78%'))

const columns = computed(() =>
  props.days.map((day) => ({
    day,
    isWeekend: day.date.getDay() === 0 || day.date.getDay() === 6,
    weekday: day.date.toLocaleDateString(undefined, { weekday: 'short' }),
    dayNumber: day.date.getDate(),
    label: `${day.date.toLocaleDateString(undefined, { weekday: 'long', day: 'numeric', month: 'long' })}: ${formatDuration(day.totalSeconds)}`,
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
        <span class="absolute right-1 top-0.5 bg-surface px-1.5 font-mono text-[9.5px] text-text-muted">
          8h
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
        <span
          v-if="column.isWeekend"
          class="pointer-events-none absolute inset-x-0 bottom-6 top-0 rounded-t-[3px] bg-canvas/80"
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
          class="relative transition-[filter] duration-150 group-hover:brightness-110"
          :class="index === 0 ? 'rounded-t-[4px]' : ''"
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
          class="pointer-events-none absolute inset-x-0 -top-1 text-center font-mono text-[10px] text-text opacity-0 transition-opacity group-hover:opacity-100"
          aria-hidden="true"
        >
          {{ formatDuration(column.day.totalSeconds) }}
        </span>

        <span
          class="absolute inset-x-0 bottom-0 truncate text-center text-[10px] leading-6"
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
