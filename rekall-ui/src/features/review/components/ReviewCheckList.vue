<script setup lang="ts">
import { runOutcome } from '@/features/check/lib/check-results'
import type { CheckResult } from '@/features/check/model/check'

/** The last run of each check on a claim. A failed run shows the end of its output, since that is what the decision turns on. */
defineProps<{ runs: readonly CheckResult[] }>()
</script>

<template>
  <ul class="flex flex-col" data-testid="review-checks">
    <li v-for="run in runs" :key="run.id" data-testid="review-check" :data-passed="run.passed">
      <div class="flex min-h-7.5 items-center gap-2.5 text-small" :class="run.passed ? 'text-text-muted' : 'text-danger'">
        <svg class="size-3.5 shrink-0" :class="run.passed && 'text-safe'" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path v-if="run.passed" d="M2.6 7.4 5.6 10.4 11.4 4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
          <path v-else d="M3.5 3.5l7 7M10.5 3.5l-7 7" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" />
        </svg>
        <span class="min-w-0 flex-1 truncate" :title="run.command">{{ run.label }}</span>
        <span class="shrink-0" :class="!run.passed && 'font-medium'">{{ runOutcome(run) }}</span>
      </div>
      <pre
        v-if="!run.passed && run.outputTail.trim()"
        class="mb-1.5 ml-6 max-h-40 overflow-auto whitespace-pre-wrap break-words rounded-control border border-border bg-canvas px-2.5 py-1.5 font-mono text-caption text-text-muted"
        data-testid="review-check-output"
        tabindex="0"
      >{{ run.outputTail }}</pre>
    </li>
  </ul>
</template>
