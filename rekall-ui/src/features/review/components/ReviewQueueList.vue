<script setup lang="ts">
import { RouterLink } from 'vue-router'
import StepSeal from '@/features/steps/components/StepSeal.vue'
import { ageLabel } from '@/features/work/lib/age-label'
import { reviewItemLine } from '@/features/review/lib/review-item-line'
import { stepNumberOf } from '@/features/deviations/model/deviation'
import { useNow } from '@/composables/useNow'
import { useConsoleStore } from '@/stores/console.store'
import type { ReviewItem } from '@/features/catalog/model/review'

/** Every claim waiting for a decision, the longest waiting first; the one in view is marked. */
defineProps<{ items: readonly ReviewItem[]; current: string | null }>()

const MINUTE_MS = 60_000

const store = useConsoleStore()
const now = useNow(MINUTE_MS)

function lineOf(item: ReviewItem): string {
  const task = store.tasks.find((candidate) => candidate.id === item.taskId)
  if (!task) return ''
  const step = item.stepId ? store.steps.find((candidate) => candidate.id === item.stepId) : undefined
  return reviewItemLine(task, step ? stepNumberOf(step, store.steps.filter((other) => other.taskId === step.taskId)) : null)
}
</script>

<template>
  <nav class="flex min-h-0 flex-col overflow-y-auto border-r border-border bg-surface px-2 pb-6" aria-label="Waiting for review" data-testid="review-list">
    <div class="flex items-center gap-2.5 px-2.5 pb-2 pt-4">
      <h1 class="text-title font-semibold text-text">Review</h1>
      <span class="rounded-key bg-accent px-1.5 font-mono text-caption font-medium leading-4 text-accent-ink" data-testid="review-count">{{ items.length }}</span>
      <span class="flex-1" />
      <span class="text-caption text-text-subtle">longest waiting first</span>
    </div>
    <RouterLink
      v-for="item in items"
      :key="item.key"
      :to="{ name: 'review', params: { key: item.key } }"
      class="focus-ring grid grid-cols-[22px_minmax(0,1fr)] items-start gap-2.5 rounded-control px-2.5 py-2 transition-colors duration-(--motion-quick)"
      :class="current === item.key ? 'selected-row' : 'hover:bg-surface-hover'"
      :aria-current="current === item.key ? 'true' : undefined"
      data-testid="review-item"
    >
      <span class="mt-0.5 size-[22px]" aria-hidden="true"><StepSeal state="CLAIMED" /></span>
      <span class="min-w-0">
        <span class="block truncate text-body text-text">{{ item.stepTitle ?? item.taskTitle }}</span>
        <span class="mt-0.5 flex items-baseline gap-2 text-caption text-text-subtle">
          <span class="min-w-0 truncate">{{ lineOf(item) }}</span>
          <span class="ml-auto shrink-0 tabular-nums">{{ item.claimedAt ? ageLabel(item.claimedAt, now) : '' }}</span>
        </span>
      </span>
    </RouterLink>
  </nav>
</template>
