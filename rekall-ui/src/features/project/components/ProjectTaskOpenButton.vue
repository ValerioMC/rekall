<script setup lang="ts">
import { computed } from 'vue'
import { useOpenTask } from '@/features/task/composables/useOpenTask'
import { useWorkingSetStore } from '@/features/workingset/store/workingSet.store'
import type { TaskId } from '@/model/branded'

/** The arrow at the start of a ledger row: opens the task without opening the row first. A dot marks a task already open in the bar. */
const props = defineProps<{ taskId: TaskId; title: string }>()

const openTask = useOpenTask()
const workingSet = useWorkingSetStore()
const isOpen = computed(() => workingSet.isOpen(props.taskId))
</script>

<template>
  <button
    type="button"
    class="focus-ring relative grid size-7 cursor-pointer place-items-center rounded-control text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-hover hover:text-text group-hover/row:text-text-muted"
    :class="isOpen && 'text-text'"
    :aria-label="isOpen ? `Open ${title}, already open in the bar` : `Open ${title}`"
    :title="isOpen ? `${title} is open in the bar` : `Open ${title}`"
    data-testid="task-row-open"
    @click.stop="openTask(props.taskId)"
    @keydown.enter.stop
  >
    <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
      <path d="M2.5 7h9m-3.5-3.5L11.5 7 8 10.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
    <span v-if="isOpen" class="absolute right-1 top-1 size-1.5 rounded-full bg-text" data-testid="task-row-working-set" aria-hidden="true"></span>
  </button>
</template>
