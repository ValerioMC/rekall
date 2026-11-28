<script setup lang="ts">
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import type { SegmentOption } from '@/components/ui/SegmentedControl.vue'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import { SESSION_PROVIDERS, SESSION_PROVIDER_LABEL } from '@/features/claude/model/session-provider'
import { tokensConsumed } from '@/features/provider-report/lib/provider-report'
import { CLOSURE_CREDIT_LABEL } from '@/features/provider-report/model/provider-report'
import type { ProviderReportPeriod } from '@/features/provider-report/model/provider-report'
import type { CalendarPeriod } from '@/features/time/lib/period'
import ProviderReportToken from './ProviderReportToken.vue'

defineProps<{ periods: readonly ProviderReportPeriod[]; selectedStart?: string }>()
const grouping = defineModel<CalendarPeriod>('grouping', { required: true })
const emit = defineEmits<{ selectPeriod: [start: string] }>()
const options: readonly SegmentOption<CalendarPeriod>[] = [
  { value: 'week', label: 'Week' }, { value: 'month', label: 'Month' }, { value: 'year', label: 'Year' }
]
const closureColumns = ['BOTH', 'NOT_RECORDED', 'total'] as const
</script>

<template>
  <section aria-labelledby="provider-report-periods">
    <div class="report-section-heading">
      <h2 id="provider-report-periods" class="text-heading font-medium">By period</h2>
      <SegmentedControl v-model="grouping" :options="options" label="Period grouping" class="report-grouping" />
    </div>
    <div class="report-ledger report-period-scroll">
      <table class="report-periods" aria-labelledby="provider-report-periods">
        <colgroup><col></colgroup>
        <colgroup v-for="provider in SESSION_PROVIDERS" :key="provider" span="3"></colgroup>
        <colgroup v-for="credit in closureColumns" :key="credit" span="2"></colgroup>
        <thead>
          <tr>
            <th scope="col" rowspan="2">Period · UTC</th>
            <th v-for="provider in SESSION_PROVIDERS" :key="provider" scope="colgroup" colspan="3" class="report-boundary">
              <span class="inline-flex items-center gap-2"><SessionProviderMark :provider="provider" />{{ SESSION_PROVIDER_LABEL[provider] }}</span>
            </th>
            <th v-for="credit in closureColumns" :key="credit" scope="colgroup" colspan="2" class="report-boundary">
              {{ credit === 'total' ? 'Total closed' : CLOSURE_CREDIT_LABEL[credit] }}
            </th>
          </tr>
          <tr>
            <template v-for="provider in SESSION_PROVIDERS" :key="provider">
              <th scope="col" class="report-boundary text-right">Tokens</th>
              <th scope="col" class="text-right">Tasks</th>
              <th scope="col" class="text-right">Steps</th>
            </template>
            <template v-for="credit in closureColumns" :key="credit">
              <th scope="col" class="report-boundary text-right">Tasks</th>
              <th scope="col" class="text-right">Steps</th>
            </template>
          </tr>
        </thead>
        <tbody>
          <tr v-for="period in periods" :key="period.start" :class="{ 'is-selected': selectedStart === period.start }" :data-period-start="period.start">
            <th scope="row">
              <button
                type="button"
                class="focus-ring report-period-key"
                :aria-pressed="selectedStart === period.start"
                aria-controls="provider-report-task-detail"
                @click="emit('selectPeriod', period.start)"
              >
                <span>{{ period.label }}</span>
                <svg class="shrink-0" width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true"><path d="m6 4 4 4-4 4" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" /></svg>
              </button>
            </th>
            <template v-for="provider in SESSION_PROVIDERS" :key="provider">
              <td class="report-boundary report-number"><ProviderReportToken :count="tokensConsumed(period.tokens[provider])" /></td>
              <td class="report-number" :class="{ 'text-text-subtle': period.closures[provider].tasks === 0 }">{{ period.closures[provider].tasks }}</td>
              <td class="report-number" :class="{ 'text-text-subtle': period.closures[provider].steps === 0 }">{{ period.closures[provider].steps }}</td>
            </template>
            <template v-for="credit in closureColumns" :key="credit">
              <template v-for="measure in (['tasks', 'steps'] as const)" :key="measure">
                <td
                  class="report-number"
                  :class="{ 'report-boundary': measure === 'tasks', 'report-sum': credit === 'total', 'text-text-subtle': (credit === 'total' ? period.totalClosed : period.closures[credit])[measure] === 0 }"
                >{{ (credit === 'total' ? period.totalClosed : period.closures[credit])[measure] }}</td>
              </template>
            </template>
          </tr>
        </tbody>
      </table>
    </div>
  </section>
</template>
