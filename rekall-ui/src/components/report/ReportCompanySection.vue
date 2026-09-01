<script setup lang="ts">
import { computed } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import { identityHue } from '@/common/identity'
import { formatDuration } from '@/common/format/duration'
import { rkCommand } from '@/common/format/rk-command'
import type { ReportCompanyRow } from '@/common/report/time-report'

/**
 * One client's share of the period: what was worked on, under which project, for how long.
 *
 * The task is the row because the task is what a person did. The project is a heading rather
 * than a column, because a reader scanning for one piece of work does not want to read the same
 * project name eleven times to find it.
 */
const props = defineProps<{ company: ReportCompanyRow }>()

const emit = defineEmits<{ copyAnchor: [anchor: string] }>()

const hue = computed(() => identityHue(props.company.companyId))

/** The busiest day this company had, so a row's bars are read against its own week. */
const peak = computed(() =>
  Math.max(
    1,
    ...props.company.projects.flatMap((project) =>
      project.tasks.flatMap((task) => [...task.perDaySeconds])
    )
  )
)

function intensity(seconds: number): number {
  return seconds === 0 ? 0 : 0.25 + 0.75 * (seconds / peak.value)
}
</script>

<template>
  <AppCard :padded="false" data-testid="report-company">
    <header
      class="relative flex items-center gap-3 overflow-hidden rounded-t-[var(--radius-card)] border-b border-border px-5 py-4"
    >
      <div
        class="texture-grid pointer-events-none absolute inset-0 opacity-70"
        :style="{ '--texture-tint': hue.base }"
        aria-hidden="true"
      />
      <div class="pointer-events-none absolute inset-x-0 top-0 h-px" :style="{ background: hue.line }" aria-hidden="true" />

      <span class="relative size-2 shrink-0 rounded-full" :style="{ background: hue.base }" aria-hidden="true" />
      <div class="relative min-w-0 flex-1">
        <h2 class="truncate text-[15px] font-semibold tracking-[-0.01em] text-text">
          {{ company.name }}
        </h2>
        <p class="mt-0.5 font-mono text-[11px] text-text-subtle">
          {{ Math.round(company.share * 100) }}% of the period
        </p>
      </div>

      <div class="relative shrink-0 text-right">
        <p class="font-mono text-[19px] font-semibold tracking-[-0.02em] text-text tabular-nums">
          {{ formatDuration(company.totalSeconds) }}
        </p>
        <div class="mt-1.5 h-1 w-28 overflow-hidden rounded-full bg-canvas">
          <div
            class="h-full rounded-full"
            :style="{ width: `${Math.max(2, company.share * 100)}%`, background: hue.base }"
          />
        </div>
      </div>
    </header>

    <div v-for="project in company.projects" :key="project.projectId" class="border-b border-border last:border-b-0">
      <div class="flex items-baseline gap-2 px-5 pb-1.5 pt-3.5">
        <h3 class="truncate text-[12.5px] font-medium text-text-muted">{{ project.title }}</h3>
        <code class="truncate font-mono text-[10.5px] text-anchor/70">project:{{ project.label }}</code>
        <span class="ml-auto shrink-0 font-mono text-[11.5px] text-text-subtle tabular-nums">
          {{ formatDuration(project.totalSeconds) }}
        </span>
      </div>

      <ul>
        <li
          v-for="task in project.tasks"
          :key="task.taskId"
          class="group flex items-center gap-3 px-5 py-2 transition-colors hover:bg-surface-raised"
          data-testid="report-task"
        >
          <div class="min-w-0 flex-1">
            <p class="truncate text-[13.5px] text-text">
              {{ task.title }}
              <span
                v-if="task.isRunning"
                class="ml-1.5 align-middle font-mono text-[10px] text-accent"
                data-testid="report-running"
              >
                running
              </span>
            </p>
            <button
              class="focus-ring mt-0.5 truncate font-mono text-[10.5px] text-anchor/70 transition-colors hover:text-anchor"
              :title="`Copy ${rkCommand(task.anchor)}`"
              data-testid="report-copy-anchor"
              @click="emit('copyAnchor', task.anchor)"
            >
              {{ task.anchor }}
            </button>
          </div>

          <!-- The same days as the ridge above, at row scale: where in the period this one went. -->
          <div
            class="hidden h-4 w-[176px] shrink-0 items-end sm:flex"
            :class="task.perDaySeconds.length > 14 ? 'gap-px' : 'gap-[2px]'"
            aria-hidden="true"
          >
            <span
              v-for="(seconds, index) in task.perDaySeconds"
              :key="index"
              class="h-full flex-1 rounded-[1px]"
              :style="{
                background: seconds === 0 ? 'var(--color-border)' : hue.base,
                opacity: seconds === 0 ? 0.5 : intensity(seconds)
              }"
            />
          </div>

          <span class="w-16 shrink-0 text-right font-mono text-[13px] text-text tabular-nums">
            {{ formatDuration(task.totalSeconds) }}
          </span>
        </li>
      </ul>
    </div>
  </AppCard>
</template>
