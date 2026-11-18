<script setup lang="ts">
import { computed } from 'vue'
import StatusMixBar from '@/components/ui/StatusMixBar.vue'
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL, TASK_STATUS_ORDER } from '@/features/catalog/model/task-status'
import type { Task } from '@/features/catalog/model/catalog'
import type { EpicProgress } from '@/features/catalog/model/epic'

/** How far the epic is: tasks done, the mix of their statuses, the steps behind them, and what waits on you. */
const props = defineProps<{ progress: EpicProgress; children: readonly Task[] }>()

const percent = computed(() => (props.progress.total ? Math.round((props.progress.done / props.progress.total) * 100) : 0))
const isComplete = computed(() => props.progress.total > 0 && props.progress.done === props.progress.total)

const legend = computed(() =>
  TASK_STATUS_ORDER.map((status) => ({ status, count: props.children.filter((task) => task.status === status).length })).filter(
    (entry) => entry.count > 0
  )
)
</script>

<template>
  <section class="shrink-0 border-b border-border px-5 py-4" aria-label="Progress" data-testid="epic-progress-panel">
    <p class="flex items-baseline gap-2">
      <span class="font-mono text-title font-semibold tabular-nums text-text">{{ progress.done }} of {{ progress.total }}</span>
      <span class="text-small text-text-muted">tasks done</span>
      <span class="ml-auto font-mono text-small font-semibold tabular-nums" :class="isComplete ? 'text-safe' : 'text-epic'">{{ percent }}%</span>
    </p>

    <StatusMixBar class="mt-3" :tasks="children" size="md" />

    <ul v-if="legend.length" class="mt-2.5 flex flex-wrap gap-x-3.5 gap-y-1" aria-label="Tasks by status">
      <li v-for="entry in legend" :key="entry.status" class="flex items-center gap-1.5 text-caption text-text-muted">
        <span class="size-[7px] rounded-full" :class="TASK_STATUS_COLOR[entry.status]" aria-hidden="true" />
        {{ TASK_STATUS_LABEL[entry.status] }}
        <span class="font-mono tabular-nums text-text-subtle">{{ entry.count }}</span>
      </li>
    </ul>

    <p class="mt-3.5 flex gap-4 border-t border-border pt-3 text-small text-text-muted">
      <span>
        <span class="mr-1 font-mono text-body font-semibold tabular-nums text-text">{{ progress.stepsDone }}/{{ progress.stepCount }}</span>
        steps done
      </span>
      <span v-if="progress.awaitingReview > 0" class="text-accent" data-testid="epic-awaiting-review">
        <span class="mr-1 font-mono text-body font-semibold tabular-nums">{{ progress.awaitingReview }}</span>
        to review
      </span>
    </p>
  </section>
</template>
