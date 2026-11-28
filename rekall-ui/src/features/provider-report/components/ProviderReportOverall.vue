<script setup lang="ts">
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import { SESSION_PROVIDERS, SESSION_PROVIDER_LABEL } from '@/features/claude/model/session-provider'
import { tokensConsumed } from '@/features/provider-report/lib/provider-report'
import { CLOSURE_CREDITS } from '@/features/provider-report/model/provider-report'
import type { ProviderReportFigures, ProviderReportPeriod } from '@/features/provider-report/model/provider-report'
import type { CalendarPeriod } from '@/features/time/lib/period'
import type { TokenTotals } from '@/features/tokens/model/token-usage'
import ProviderReportToken from './ProviderReportToken.vue'

defineProps<{ figures: ProviderReportFigures; periods: readonly ProviderReportPeriod[]; selectedStart?: string }>()
const scope = defineModel<CalendarPeriod | 'all'>('scope', { required: true })
const emit = defineEmits<{ selectPeriod: [start: string] }>()
const kinds: readonly Readonly<{ value: keyof TokenTotals; label: string }>[] = [
  { value: 'input', label: 'Input' }, { value: 'cacheWrite', label: 'Cache write' },
  { value: 'cacheRead', label: 'Cache read' }, { value: 'generated', label: 'Output' }
]
const measures = [{ value: 'tasks', label: 'Tasks closed' }, { value: 'steps', label: 'Steps closed' }] as const

function choosePeriod(event: Event): void {
  if (event.target instanceof HTMLSelectElement) emit('selectPeriod', event.target.value)
}
</script>

<template>
  <section aria-labelledby="provider-report-overall">
    <div class="report-section-heading">
      <h2 id="provider-report-overall" class="text-heading font-medium">Overall</h2>
      <div class="flex items-center gap-2" role="group" aria-label="Overall date range">
        <select v-model="scope" class="focus-ring report-scope-select" aria-label="Overall grouping">
          <option value="all">All time</option>
          <option value="week">Week</option>
          <option value="month">Month</option>
          <option value="year">Year</option>
        </select>
        <select v-if="scope !== 'all'" :value="selectedStart" class="focus-ring report-scope-select" aria-label="Overall period (UTC)" @change="choosePeriod">
          <option v-for="period in periods" :key="period.start" :value="period.start">{{ period.label }}</option>
        </select>
      </div>
    </div>
    <div class="report-ledger">
      <table class="report-totals" aria-labelledby="provider-report-overall">
        <colgroup><col><col><col><col><col><col></colgroup>
        <thead>
          <tr>
            <th scope="col">Measure</th>
            <th v-for="provider in SESSION_PROVIDERS" :key="provider" scope="col" class="report-boundary">
              <span class="inline-flex items-center gap-2"><SessionProviderMark :provider="provider" />{{ SESSION_PROVIDER_LABEL[provider] }}</span>
            </th>
            <th scope="col" class="report-boundary text-right">Both</th>
            <th scope="col" class="text-right">Not recorded</th>
            <th scope="col" class="report-boundary text-right">Total</th>
          </tr>
        </thead>
        <tbody>
          <tr class="report-token-summary">
            <th scope="row">Tokens consumed</th>
            <td v-for="provider in SESSION_PROVIDERS" :key="provider" class="report-boundary" :data-provider="provider">
              <ProviderReportToken class="report-token-total" :count="tokensConsumed(figures.tokens[provider])" />
              <dl class="report-token-parts">
                <div v-for="kind in kinds" :key="kind.value" class="flex items-baseline justify-between gap-1.5">
                  <dt class="whitespace-nowrap text-text-muted">{{ kind.label }}</dt>
                  <dd class="report-number"><ProviderReportToken :count="figures.tokens[provider][kind.value]" /></dd>
                </div>
              </dl>
            </td>
            <td class="report-boundary report-number text-text-subtle" aria-label="Not applicable">—</td>
            <td class="report-number text-text-subtle" aria-label="Not applicable">—</td>
            <td class="report-boundary report-sum text-right">
              <ProviderReportToken class="report-token-total" :count="tokensConsumed(figures.tokens.CLAUDE) + tokensConsumed(figures.tokens.CODEX)" />
            </td>
          </tr>
          <tr v-for="measure in measures" :key="measure.value" class="text-title">
            <th scope="row" class="text-body">{{ measure.label }}</th>
            <td v-for="credit in CLOSURE_CREDITS" :key="credit" class="report-number" :class="{ 'report-boundary': credit !== 'NOT_RECORDED', 'text-text-subtle': figures.closures[credit][measure.value] === 0 }">
              {{ figures.closures[credit][measure.value] }}
            </td>
            <td class="report-boundary report-number report-sum" :class="{ 'text-text-subtle': figures.totalClosed[measure.value] === 0 }">{{ figures.totalClosed[measure.value] }}</td>
          </tr>
        </tbody>
      </table>
      <p class="border-t border-border px-4 py-1.5 text-small text-text-muted">Each closure counts once. Credit comes from the providers with recorded tokens on the task or step.</p>
    </div>
  </section>
</template>
