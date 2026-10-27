<script setup lang="ts">
import { computed } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import StatusMixBar from '@/components/ui/StatusMixBar.vue'
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL, TASK_STATUS_ORDER } from '@/features/catalog/model/catalog'
import type { Task } from '@/features/catalog/model/catalog'
import type { EpicProgress } from '@/features/catalog/model/epic'

/** How far the epic is: the share of its tasks done, the mix of their statuses, and what waits on you. */
const props = defineProps<{ progress: EpicProgress; children: readonly Task[] }>()

const percent = computed(() => (props.progress.total ? Math.round((props.progress.done / props.progress.total) * 100) : 0))

const legend = computed(() =>
  TASK_STATUS_ORDER.map((status) => ({ status, count: props.children.filter((task) => task.status === status).length })).filter(
    (entry) => entry.count > 0
  )
)

const tiles = computed(() => [
  { key: 'steps', label: 'Steps done', value: `${props.progress.stepsDone}/${props.progress.stepCount}`, tone: 'text-text' },
  {
    key: 'review',
    label: 'Awaiting your review',
    value: String(props.progress.awaitingReview),
    tone: props.progress.awaitingReview ? 'text-accent' : 'text-text'
  },
  {
    key: 'live',
    label: 'In progress',
    value: String(props.children.filter((task) => task.status === 'IN_PROGRESS').length),
    tone: 'text-text'
  }
])
</script>

<template>
  <AppCard data-testid="epic-progress-panel">
    <div class="flex flex-wrap items-end gap-x-6 gap-y-2">
      <div>
        <p class="eyebrow text-caption">Progress</p>
        <p class="mt-1 flex items-baseline gap-2">
          <span class="font-mono text-hero font-semibold leading-none tabular-nums text-text">{{ progress.done }}</span>
          <span class="font-mono text-title tabular-nums text-text-subtle">/ {{ progress.total }}</span>
          <span class="text-small text-text-muted">tasks done</span>
        </p>
      </div>
      <span
        class="ml-auto font-mono text-title font-semibold tabular-nums"
        :class="progress.total && progress.done === progress.total ? 'text-safe' : 'text-epic'"
      >
        {{ percent }}%
      </span>
    </div>

    <StatusMixBar class="mt-4" :tasks="children" size="md" />

    <ul v-if="legend.length" class="mt-2.5 flex flex-wrap gap-x-4 gap-y-1" aria-label="Tasks by status">
      <li v-for="entry in legend" :key="entry.status" class="flex items-center gap-1.5 text-caption text-text-muted">
        <span class="size-[7px] rounded-full" :class="TASK_STATUS_COLOR[entry.status]" aria-hidden="true" />
        {{ TASK_STATUS_LABEL[entry.status] }}
        <span class="font-mono tabular-nums text-text-subtle">{{ entry.count }}</span>
      </li>
    </ul>

    <dl class="mt-5 grid grid-cols-3 gap-2">
      <div v-for="tile in tiles" :key="tile.key" class="rounded-control border border-border bg-canvas px-3 py-2.5">
        <dt class="truncate text-micro text-text-subtle">{{ tile.label }}</dt>
        <dd class="mt-0.5 font-mono text-title font-semibold tabular-nums" :class="tile.tone">{{ tile.value }}</dd>
      </div>
    </dl>
  </AppCard>
</template>
