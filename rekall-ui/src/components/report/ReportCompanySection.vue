<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import { identityHue } from '@/common/identity'
import { formatDuration } from '@/common/format/duration'
import { rkCommand } from '@/common/format/rk-command'
import { stepDay, stepTail } from '@/common/report/time-report'
import type { ReportCompanyRow, ReportTaskRow } from '@/common/report/time-report'
import type { ReportPeriod } from '@/common/report/period'
import type { TaskId } from '@/model/branded'

/**
 * One client's share of the period: what was worked on, under which project, for how long, and
 * what came out of it.
 *
 * The task is the row because the task is what a person did. The project is a heading rather
 * than a column, because a reader scanning for one piece of work does not want to read the same
 * project name eleven times to find it.
 *
 * Under the row are the steps ticked inside the period. Hours are the invoice and the steps are
 * the answer to the question anyone reading the invoice asks next, so they are part of the row
 * rather than a screen someone has to go and find.
 */
const props = defineProps<{
  company: ReportCompanyRow
  period: ReportPeriod
  /** What the header toggle says. A row can still disagree with it, until it is flipped again. */
  stepsOpen: boolean
}>()

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

// ------------------------------------------------------------------ the steps under a row

/**
 * Rows the reader has folded away from, or unfolded towards, whatever the header says.
 *
 * Held as the exceptions rather than as the state of every row: the toggle above is the
 * intent, and flipping it clears the exceptions, because that is what pressing it means.
 */
const overrides = ref<ReadonlySet<TaskId>>(new Set())

watch(
  () => props.stepsOpen,
  () => (overrides.value = new Set())
)

function hasSteps(task: ReportTaskRow): boolean {
  return task.closedSteps.length + task.openStepCount + task.doneElsewhereCount > 0
}

function isOpen(task: ReportTaskRow): boolean {
  return overrides.value.has(task.taskId) ? !props.stepsOpen : props.stepsOpen
}

function toggleSteps(task: ReportTaskRow): void {
  const next = new Set(overrides.value)
  if (next.has(task.taskId)) next.delete(task.taskId)
  else next.add(task.taskId)
  overrides.value = next
}

/**
 * The line under the steps: what the checklist still holds that this period did not close.
 *
 * A task that closed nothing says so outright. Reading hours against an empty list and being
 * left to wonder whether the steps are missing or the work simply did not finish anything is
 * the one thing this line exists to prevent.
 */
function footnote(task: ReportTaskRow): string | null {
  const tail = stepTail(task)
  if (task.closedSteps.length > 0) return tail
  const nothing = `Nothing closed this ${props.period}`
  return tail ? `${nothing} · ${tail}` : nothing
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
          class="group transition-colors hover:bg-surface-raised"
          data-testid="report-task"
        >
          <div class="flex items-center gap-3 px-5 py-2">
            <!-- The count is what this task closed in the period, which is the number the row is
                 being read for. It sits on the control that opens the list rather than beside the
                 title, so one glance down the column says which tasks moved. -->
            <button
              v-if="hasSteps(task)"
              class="focus-ring flex h-6 shrink-0 items-center gap-1 rounded-[6px] pl-0.5 pr-1 transition-colors"
              :aria-expanded="isOpen(task)"
              :aria-label="`Steps closed on ${task.title}`"
              data-testid="report-steps-toggle-task"
              @click="toggleSteps(task)"
            >
              <svg
                class="size-3 text-text-subtle transition-transform duration-200"
                :class="isOpen(task) && 'rotate-90'"
                viewBox="0 0 24 24"
                fill="none"
                aria-hidden="true"
              >
                <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
              <span
                class="font-mono text-[11px] tabular-nums"
                :style="task.closedSteps.length > 0 ? { color: hue.base } : undefined"
                :class="task.closedSteps.length === 0 && 'text-text-subtle'"
              >
                {{ task.closedSteps.length }}
              </span>
            </button>
            <span v-else class="h-6 w-[26px] shrink-0" aria-hidden="true" />

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
          </div>

          <!-- What the hours produced, in the order it happened. The same node-on-a-line the
               checklist itself is drawn as, so a step read here is recognisably the step that
               was ticked there. -->
          <div
            v-if="hasSteps(task) && isOpen(task)"
            class="max-w-[720px] pb-2.5 pl-[42px] pr-5"
            data-testid="report-task-steps"
          >
            <ol v-if="task.closedSteps.length" class="relative">
              <span
                v-if="task.closedSteps.length > 1"
                class="absolute bottom-[11px] left-[3px] top-[11px] w-px"
                :style="{ background: hue.line }"
                aria-hidden="true"
              />
              <li
                v-for="step in task.closedSteps"
                :key="step.stepId"
                class="relative flex items-baseline gap-3 py-[2px] pl-4"
                data-testid="report-step"
              >
                <span
                  class="absolute left-0 top-[7px] size-[7px] rounded-full"
                  :style="{ background: hue.base }"
                  aria-hidden="true"
                />
                <span class="min-w-0 flex-1 truncate text-[12.5px] leading-relaxed text-text-muted">
                  {{ step.title }}
                </span>
                <span class="shrink-0 font-mono text-[10.5px] tabular-nums text-text-subtle">
                  {{ stepDay(step.doneAt) }}
                </span>
              </li>
            </ol>

            <p
              v-if="footnote(task)"
              class="pl-4 text-[11.5px] leading-relaxed text-text-subtle"
              :class="task.closedSteps.length > 0 && 'mt-1'"
              data-testid="report-step-footnote"
            >
              {{ footnote(task) }}
            </p>
          </div>
        </li>
      </ul>
    </div>
  </AppCard>
</template>
