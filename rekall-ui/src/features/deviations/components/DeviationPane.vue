<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import TaskTitleRow from '@/features/catalog/components/TaskTitleRow.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useTaskDeviations } from '@/features/deviations/composables/useTaskDeviations'
import { neighbourOf } from '@/features/deviations/model/deviation'
import { relativeTime } from '@/common/format/relative-time'

/**
 * One deviation of the task in view, read in the main pane. The links above the text walk the
 * task's other deviations, and every move is a history entry, so the browser's arrows go back and
 * forth between them as they do between panes.
 */
const store = useConsoleStore()
const { selectedTask, selectedTaskSteps, selectedDeviationStep } = storeToRefs(store)

const { deviations, isLoaded } = useTaskDeviations(computed(() => selectedTask.value?.id ?? null))

const current = computed(() => deviations.value.find((deviation) => deviation.stepNumber === selectedDeviationStep.value) ?? null)
const previous = computed(() => neighbourOf(deviations.value, selectedDeviationStep.value ?? 0, -1))
const next = computed(() => neighbourOf(deviations.value, selectedDeviationStep.value ?? 0, 1))
const position = computed(() => deviations.value.findIndex((deviation) => deviation.stepNumber === selectedDeviationStep.value) + 1)

const stepOfDeviation = computed(() => {
  const checklist = selectedTaskSteps.value.filter((step) => step.state !== 'DRAFT')
  return checklist[(selectedDeviationStep.value ?? 0) - 1] ?? null
})

const wasRewritten = computed(() => current.value !== null && current.value.updatedAt !== current.value.createdAt)

function go(by: 1 | -1): void {
  const target = by === 1 ? next.value : previous.value
  if (target) store.openDeviation(target.stepNumber)
}

function openStep(): void {
  if (selectedTask.value && stepOfDeviation.value) store.openStep(selectedTask.value.id, stepOfDeviation.value.id)
}

defineExpose({ step: go })
</script>

<template>
  <section class="flex min-h-0 flex-1 flex-col bg-canvas" aria-label="Deviation" data-testid="deviation-pane">
    <p v-if="!selectedTask" class="px-9 py-12 text-body text-text-muted">Pick a task to see where its work left the plan.</p>

    <template v-else>
      <header class="flex shrink-0 items-start gap-3.5 border-b border-border px-5 py-3.5">
        <div class="min-w-0 flex-1">
          <p class="eyebrow flex items-center gap-1.5">
            <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
            Deviation
          </p>
          <TaskTitleRow :task="selectedTask" copy-testid="copy-deviation-anchor" />
        </div>

        <div v-if="deviations.length > 1" class="flex shrink-0 items-center gap-1" role="group" aria-label="Deviations of this task">
          <AppButton
            variant="quiet"
            size="icon"
            :disabled="!previous"
            :title="previous ? `Step ${previous.stepNumber} ( [ )` : 'This is the first deviation'"
            aria-label="Previous deviation"
            data-testid="deviation-previous"
            @click="go(-1)"
          >
            <svg class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
              <path d="M7.5 2.5 4 6l3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </AppButton>
          <span class="min-w-[3.25rem] text-center font-mono text-caption tabular-nums text-text-muted" data-testid="deviation-position">
            {{ position }} of {{ deviations.length }}
          </span>
          <AppButton
            variant="quiet"
            size="icon"
            :disabled="!next"
            :title="next ? `Step ${next.stepNumber} ( ] )` : 'This is the last deviation'"
            aria-label="Next deviation"
            data-testid="deviation-next"
            @click="go(1)"
          >
            <svg class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
              <path d="M4.5 2.5 8 6l-3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </AppButton>
        </div>

        <AppButton variant="quiet" size="sm" data-testid="deviation-close" title="Back to the notes (Esc)" @click="store.closeDeviation()">
          Close
        </AppButton>
      </header>

      <nav
        v-if="deviations.length > 1"
        class="flex shrink-0 items-center gap-x-3 overflow-x-auto border-b border-border bg-surface px-5 py-2"
        aria-label="Steps the work deviated on"
      >
        <button
          v-for="deviation in deviations"
          :key="deviation.id"
          type="button"
          class="focus-ring shrink-0 rounded-key px-1.5 py-0.5 text-caption transition-colors"
          :class="
            deviation.stepNumber === selectedDeviationStep
              ? 'bg-accent-soft font-semibold text-accent'
              : 'text-text-muted hover:text-text'
          "
          :title="deviation.stepTitle ?? undefined"
          :aria-current="deviation.stepNumber === selectedDeviationStep"
          data-testid="deviation-rail-link"
          @click="store.openDeviation(deviation.stepNumber)"
        >
          Step {{ deviation.stepNumber }}
        </button>
      </nav>

      <div v-if="current" class="min-h-0 flex-1 overflow-y-auto">
        <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1 px-5 pb-1 pt-4">
          <h3 class="min-w-0 text-heading font-semibold tracking-[-0.015em] text-text" data-testid="deviation-heading">
            Step {{ current.stepNumber }}<template v-if="current.stepTitle">: {{ current.stepTitle }}</template>
          </h3>
          <button
            v-if="stepOfDeviation"
            type="button"
            class="focus-ring rounded-key text-caption text-accent underline decoration-accent/40 underline-offset-2 hover:decoration-accent"
            data-testid="deviation-open-step"
            @click="openStep"
          >
            Open the step
          </button>
          <span v-else class="text-caption text-text-subtle">
            The checklist has no step {{ current.stepNumber }} any more.
          </span>
        </div>
        <p class="px-5 text-caption text-text-subtle">
          Recorded {{ relativeTime(current.createdAt) }}<template v-if="wasRewritten"> &middot; rewritten {{ relativeTime(current.updatedAt) }}</template>
        </p>
        <div class="p-4">
          <AppMarkdownEditor :model-value="current.bodyMarkdown" readonly />
        </div>
      </div>

      <div v-else-if="isLoaded" class="min-h-0 flex-1 overflow-y-auto">
        <div class="max-w-[560px] px-9 py-12">
          <h2 class="mb-1.5 text-display font-semibold leading-tight tracking-[-0.025em] text-text">
            <template v-if="deviations.length">No deviation on step {{ selectedDeviationStep }}</template>
            <template v-else>The work followed the plan</template>
          </h2>
          <p class="text-body leading-relaxed text-text-muted">
            A session records a deviation when it builds something other than what a step drew, with what was
            planned, what was built and why. They are kept apart from the notes and listed on the task.
          </p>
        </div>
      </div>
    </template>
  </section>
</template>
