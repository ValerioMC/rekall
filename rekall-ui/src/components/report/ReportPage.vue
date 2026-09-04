<script setup lang="ts">
import { computed, ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCatalogHeader from '@/components/catalog/AppCatalogHeader.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import ReportRidge from '@/components/report/ReportRidge.vue'
import ReportCompanySection from '@/components/report/ReportCompanySection.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import { useNow } from '@/composables/useNow'
import { identityHue } from '@/common/identity'
import { formatDuration } from '@/common/format/duration'
import { rkCommand } from '@/common/format/rk-command'
import { periodRange, shiftAnchor } from '@/common/report/period'
import { buildTimeReport, reportAsMarkdown } from '@/common/report/time-report'
import type { ReportPeriod } from '@/common/report/period'
import type { CompanyId } from '@/model/branded'

/**
 * What the week went to, by client.
 *
 * The question this answers is the one asked at the end of a week and again at the end of a
 * month: what did I do for them, and how long did it take. So the period is the frame, the
 * client is the section, and the task is the row, because the task is the unit of work someone
 * outside this application recognises.
 *
 * Everything is read from the sessions the timer already writes. Nothing here is entered by
 * hand, which is the only reason a report like this is ever true.
 */
const store = useConsoleStore()
const toast = useToastStore()
/** Half a minute is close enough for a report, and stops a running session redrawing it every
 *  second while somebody reads it. */
const now = useNow(30_000)

const period = ref<ReportPeriod>('week')
const anchor = ref(new Date())
const selected = ref<ReadonlySet<CompanyId>>(new Set())

/**
 * Whether the rows open on what they closed.
 *
 * On, because the hours alone are what a report used to be and the steps are why anyone opens
 * this screen twice. Off is for the month someone is scanning for a number.
 */
const showSteps = ref(true)

const range = computed(() => periodRange(period.value, anchor.value))

const report = computed(() =>
  buildTimeReport(
    store.timeEntries,
    store.tasks,
    store.steps,
    store.companies,
    range.value,
    now.value,
    selected.value
  )
)

/** Every company that has time in this period, filter or no filter, so the chips do not vanish. */
const wholePeriod = computed(() =>
  buildTimeReport(
    store.timeEntries,
    store.tasks,
    store.steps,
    store.companies,
    range.value,
    now.value
  )
)

const chips = computed(() =>
  wholePeriod.value.companies.map((company) => ({
    id: company.companyId,
    name: company.name,
    totalSeconds: company.totalSeconds,
    hue: identityHue(company.companyId),
    on: selected.value.size === 0 || selected.value.has(company.companyId)
  }))
)

const isCurrent = computed(() => {
  const current = periodRange(period.value, new Date(now.value))
  return current.start.getTime() === range.value.start.getTime()
})

const emptiness = computed<'none-tracked' | 'none-selected' | null>(() => {
  if (report.value.companies.length > 0) return null
  return wholePeriod.value.companies.length > 0 ? 'none-selected' : 'none-tracked'
})

function shift(delta: number): void {
  anchor.value = shiftAnchor(period.value, anchor.value, delta)
}

function goToCurrent(): void {
  anchor.value = new Date()
}

/** Switching the frame keeps the day in view, so a week in August stays in August. */
function setPeriod(next: ReportPeriod): void {
  period.value = next
}

function toggleCompany(id: CompanyId): void {
  const next = new Set(selected.value)
  if (next.has(id)) next.delete(id)
  else next.add(id)
  // Everything selected and nothing selected are the same report, and the empty set is the one
  // that keeps saying "all of them" as companies come and go.
  selected.value = next.size === wholePeriod.value.companies.length ? new Set() : next
}

function clearCompanies(): void {
  selected.value = new Set()
}

async function copyReport(): Promise<void> {
  await navigator.clipboard?.writeText(reportAsMarkdown(report.value, range.value))
  toast.notify('Report copied as markdown.')
}

async function copyAnchor(taskAnchor: string): Promise<void> {
  await navigator.clipboard?.writeText(rkCommand(taskAnchor))
  toast.notify(`Copied ${rkCommand(taskAnchor)}`)
}
</script>

<template>
  <div class="min-h-full bg-canvas">
    <AppCatalogHeader title="Report">
      <template #actions>
        <div class="flex shrink-0 gap-0.5 rounded-[8px] bg-canvas p-0.5" role="group" aria-label="Period">
          <button
            v-for="option in (['week', 'month'] as const)"
            :key="option"
            class="focus-ring flex h-7 items-center rounded-[6px] px-3 text-[12px] capitalize transition-colors"
            :class="period === option ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]' : 'text-text-subtle hover:text-text'"
            :aria-pressed="period === option"
            :data-testid="`report-period-${option}`"
            @click="setPeriod(option)"
          >
            {{ option }}
          </button>
        </div>
        <button
          class="focus-ring flex h-7 shrink-0 items-center gap-1.5 rounded-[8px] border px-2.5 text-[12px] transition-colors"
          :class="
            showSteps
              ? 'border-border-strong bg-surface-raised text-text'
              : 'border-border bg-canvas text-text-subtle hover:text-text'
          "
          :aria-pressed="showSteps"
          data-testid="report-steps-toggle"
          @click="showSteps = !showSteps"
        >
          <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M1.2 3.4 2.6 4.8l2.4-2.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
            <path d="M6.8 3.6h4M6.8 8.4h4M1.4 8.4h3.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
          </svg>
          Steps
        </button>
        <AppButton size="sm" variant="secondary" data-testid="report-copy" @click="copyReport">
          Copy as markdown
        </AppButton>
      </template>
    </AppCatalogHeader>

    <div class="mx-auto flex max-w-[1240px] flex-col gap-5 px-8 py-6">
      <AppCard :padded="false">
        <div class="flex flex-wrap items-end justify-between gap-4 px-5 pb-4 pt-5">
          <div class="min-w-0">
            <p class="eyebrow">
              Tracked this {{ period }}
            </p>
            <p
              class="mt-1 font-mono text-[34px] font-semibold leading-none tracking-[-0.03em] text-text tabular-nums"
              data-testid="report-total"
            >
              {{ formatDuration(report.totalSeconds) }}
            </p>
            <p class="mt-2 text-[12.5px] text-text-muted">
              {{ report.taskCount }} {{ report.taskCount === 1 ? 'task' : 'tasks' }},
              {{ report.companies.length }}
              {{ report.companies.length === 1 ? 'company' : 'companies' }}
              <template v-if="report.closedStepCount > 0">
                · {{ report.closedStepCount }}
                {{ report.closedStepCount === 1 ? 'step' : 'steps' }} closed
              </template>
              <template v-if="report.busiestDay && report.busiestDay.totalSeconds > 0">
                · {{ formatDuration(report.busiestDay.totalSeconds) }} on the busiest day
              </template>
            </p>
          </div>

          <div class="flex items-center gap-2">
            <button
              class="focus-ring grid size-8 place-items-center rounded-[var(--radius-control)] border border-border-strong bg-surface-raised text-text-subtle transition-colors hover:border-accent hover:bg-surface-hover hover:text-text"
              aria-label="Previous period"
              data-testid="report-prev"
              @click="shift(-1)"
            >
              <svg class="size-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                <path d="M15 6l-6 6 6 6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </button>
            <p
              class="min-w-[168px] text-center font-mono text-[13px] text-text tabular-nums"
              data-testid="report-range"
            >
              {{ range.label }}
            </p>
            <button
              class="focus-ring grid size-8 place-items-center rounded-[var(--radius-control)] border border-border-strong bg-surface-raised text-text-subtle transition-colors hover:border-accent hover:bg-surface-hover hover:text-text"
              aria-label="Next period"
              data-testid="report-next"
              @click="shift(1)"
            >
              <svg class="size-4" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
            </button>
            <AppButton
              v-if="!isCurrent"
              size="sm"
              variant="ghost"
              data-testid="report-current"
              @click="goToCurrent"
            >
              This {{ period }}
            </AppButton>
          </div>
        </div>

        <div class="px-5 pb-4">
          <ReportRidge :days="report.days" :period="period" />
        </div>
      </AppCard>

      <div v-if="chips.length" class="flex flex-wrap items-center gap-2" data-testid="report-filter">
        <span class="mr-1 eyebrow text-[11px]">
          Companies
        </span>
        <button
          v-for="chip in chips"
          :key="chip.id"
          class="focus-ring flex h-7 items-center gap-2 rounded-full border px-3 text-[12px] transition-colors"
          :class="
            chip.on
              ? 'border-border-strong bg-surface-raised text-text'
              : 'border-border bg-canvas text-text-subtle hover:text-text'
          "
          :aria-pressed="selected.size > 0 && selected.has(chip.id)"
          data-testid="report-company-chip"
          @click="toggleCompany(chip.id)"
        >
          <span
            class="size-2 rounded-full transition-opacity"
            :style="{ background: chip.hue.base, opacity: chip.on ? 1 : 0.35 }"
            aria-hidden="true"
          />
          {{ chip.name }}
          <span class="font-mono text-[10.5px] text-text-subtle tabular-nums">
            {{ formatDuration(chip.totalSeconds) }}
          </span>
        </button>
        <button
          v-if="selected.size > 0"
          class="focus-ring h-7 rounded-full px-2.5 text-[11.5px] text-text-subtle underline decoration-dotted transition-colors hover:text-text"
          data-testid="report-clear-filter"
          @click="clearCompanies"
        >
          all companies
        </button>
      </div>

      <ReportCompanySection
        v-for="company in report.companies"
        :key="company.companyId"
        :company="company"
        :period="period"
        :steps-open="showSteps"
        @copy-anchor="copyAnchor"
      />

      <AppEmptyState
        v-if="emptiness === 'none-tracked'"
        title="Nothing tracked in this period"
        :description="`No sessions were recorded between ${range.label}. Start a timer on a task and the hours land here.`"
      />
      <AppEmptyState
        v-else-if="emptiness === 'none-selected'"
        title="Nothing for the companies you picked"
        description="Those companies have no sessions in this period. Pick another, or show them all."
      >
        <AppButton size="sm" variant="secondary" @click="clearCompanies">All companies</AppButton>
      </AppEmptyState>
    </div>
  </div>
</template>
