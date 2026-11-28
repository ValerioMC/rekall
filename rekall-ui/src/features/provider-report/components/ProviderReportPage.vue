<script setup lang="ts">
import { computed, ref } from 'vue'
import { useNow } from '@/composables/useNow'
import { buildProviderReport } from '@/features/provider-report/lib/provider-report'
import type { ProviderReportInput, ProviderReportPeriod } from '@/features/provider-report/model/provider-report'
import type { CalendarPeriod } from '@/features/time/lib/period'
import { utcToday } from '@/features/tokens/lib/token-trend'
import { useTokenUsageStore } from '@/features/tokens/store/tokenUsage.store'
import { useConsoleStore } from '@/stores/console.store'
import ProviderReportOverall from './ProviderReportOverall.vue'
import ProviderReportPeriods from './ProviderReportPeriods.vue'
import ProviderReportTasks from './ProviderReportTasks.vue'

const catalog = useConsoleStore()
const usage = useTokenUsageStore()
const now = useNow(60_000)
const today = computed(() => utcToday(new Date(now.value)))
const input = computed<ProviderReportInput>(() => ({ tasks: catalog.tasks, steps: catalog.steps, tallies: usage.tallies, days: usage.days }))
const grouping = ref<CalendarPeriod>('month')
const scope = ref<CalendarPeriod | 'all'>('all')
const periodChoices = ref<Partial<Record<CalendarPeriod, string>>>({})
const overallChoices = ref<Partial<Record<CalendarPeriod, string>>>({})

const timeline = computed(() => buildProviderReport(input.value, grouping.value, today.value))
const overallPeriods = computed(() => {
  if (scope.value === 'all') return []
  if (scope.value === grouping.value) return timeline.value.periods
  return buildProviderReport(input.value, scope.value, today.value).periods
})

function periodFor(periods: readonly ProviderReportPeriod[], start: string | undefined): ProviderReportPeriod | undefined {
  return periods.find((period) => period.start === start) ?? periods[0]
}

const period = computed(() => periodFor(timeline.value.periods, periodChoices.value[grouping.value]))
const overallPeriod = computed(() => scope.value === 'all' ? undefined : periodFor(overallPeriods.value, overallChoices.value[scope.value]))

function chooseOverallPeriod(start: string): void {
  if (scope.value !== 'all') overallChoices.value[scope.value] = start
}
</script>

<template>
  <main class="provider-report page-scroll min-h-0 flex-1 bg-canvas">
    <div class="page-column provider-report-column">
      <header class="mb-5">
        <h1 class="text-display font-medium tracking-tight">Provider report</h1>
        <p class="mt-0.5 text-text-muted">Token usage and closures across all companies and projects.</p>
      </header>
      <ProviderReportOverall
        v-model:scope="scope"
        :figures="overallPeriod ?? timeline.overall"
        :periods="overallPeriods"
        :selected-start="overallPeriod?.start"
        @select-period="chooseOverallPeriod"
      />
      <ProviderReportPeriods
        v-model:grouping="grouping"
        :periods="timeline.periods"
        :selected-start="period?.start"
        @select-period="periodChoices[grouping] = $event"
      />
      <ProviderReportTasks v-if="period" :period="period" />
    </div>
  </main>
</template>

<style src="./provider-report.css"></style>
