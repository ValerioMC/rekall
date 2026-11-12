<script setup lang="ts">
import { computed } from 'vue'
import { relativeTime } from '@/common/format/relative-time'
import { failedRuns, latestRunsOfStep, runDuration, runOutcome } from '@/features/check/lib/check-results'
import { useChecksStore } from '@/features/check/store/checks.store'
import type { TaskStep } from '@/features/steps/model/task-step'

/**
 * What the last run of each check found on this step, so the review starts from what was already
 * tried. A failed run opens on the end of its output; a step nothing ran for shows nothing.
 */
const props = defineProps<{ step: TaskStep }>()

const store = useChecksStore()

const runs = computed(() => latestRunsOfStep(store.resultsOf(props.step.taskId), props.step.id))
const failures = computed(() => failedRuns(runs.value).length)
</script>

<template>
  <div v-if="runs.length" class="mb-3" data-testid="step-check-results">
    <span class="eyebrow">
      Checks
      <span class="font-mono" :class="failures ? 'text-danger' : 'text-safe'" data-testid="step-check-summary">
        {{ failures ? `${failures} of ${runs.length} failed` : `${runs.length} passed` }}
      </span>
    </span>
    <ul class="mt-1 flex flex-col gap-1">
      <li v-for="run in runs" :key="run.id" data-testid="step-check-run" :data-passed="run.passed">
        <details :open="!run.passed" class="rounded-control border border-border bg-canvas px-2.5 py-1.5">
          <summary class="focus-ring flex cursor-pointer list-none items-center gap-2 rounded-key text-small">
            <span
              class="inline-flex shrink-0 items-center gap-1 rounded-full px-1.5 py-px text-micro font-semibold"
              :class="run.passed ? 'bg-safe-soft text-safe' : 'bg-danger-soft text-danger'"
              data-testid="step-check-outcome"
            >
              <svg class="size-2.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                <path v-if="run.passed" d="M2.2 6.4 4.8 9l5-5.6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
                <path v-else d="M3 3l6 6M9 3 3 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
              </svg>
              {{ runOutcome(run) }}
            </span>
            <span class="min-w-0 flex-1 truncate text-text">{{ run.label }}</span>
            <span class="shrink-0 font-mono text-micro text-text-subtle">{{ runDuration(run.durationMs) }}</span>
            <span class="shrink-0 text-micro text-text-subtle">{{ relativeTime(run.ranAt) }}</span>
          </summary>
          <code class="mt-1.5 block truncate font-mono text-caption text-text-subtle" :title="run.command">{{ run.command }}</code>
          <pre
            v-if="run.outputTail.trim()"
            class="mt-1.5 max-h-60 overflow-auto whitespace-pre-wrap break-words rounded-key bg-surface-raised p-2 font-mono text-caption text-text-muted"
            data-testid="step-check-output"
            tabindex="0"
          >{{ run.outputTail }}</pre>
          <p v-else class="mt-1.5 text-caption text-text-subtle">It printed nothing.</p>
        </details>
      </li>
    </ul>
  </div>
</template>

<style scoped>
summary::-webkit-details-marker {
  display: none;
}
</style>
