<script setup lang="ts">
import { useOpenTask } from '@/features/work/composables/useOpenTask'
import type { TaskId } from '@/model/branded'

/** The arrow at the end of a ledger row: opens the task without opening the row first. */
const props = defineProps<{ taskId: TaskId; title: string }>()

const openTask = useOpenTask()
</script>

<template>
  <button
    type="button"
    class="focus-ring grid size-7 cursor-pointer place-items-center rounded-control text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-hover hover:text-text group-hover/row:text-text-muted"
    :aria-label="`Open ${title}`"
    :title="`Open ${title}`"
    data-testid="work-row-open"
    @click.stop="openTask(props.taskId)"
    @keydown.enter.stop
  >
    <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
      <path d="M2.5 7h9m-3.5-3.5L11.5 7 8 10.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </button>
</template>
