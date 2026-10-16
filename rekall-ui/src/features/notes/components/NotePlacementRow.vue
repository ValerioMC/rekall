<script setup lang="ts">
import { computed } from 'vue'
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL } from '@/features/catalog/model/catalog'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * One task in the new-note composer's picker: the whole row is a button whose tick says whether
 * the note will be on the task, and a click flips it.
 */
const props = withDefaults(
  defineProps<{
    task: Task
    attached: boolean
    walkIndex: number
    highlighted?: boolean
  }>(),
  { highlighted: false }
)

const emit = defineEmits<{ toggle: []; hover: [] }>()

const pickTitle = computed(() =>
  props.attached ? `Take this note off ${props.task.title}` : `Put this note on ${props.task.title}`
)
</script>

<template>
  <button
    type="button"
    class="group/row focus-ring flex w-full cursor-pointer items-center gap-2.5 rounded-control py-1.5 pl-2 pr-2 text-left transition-colors duration-(--motion-quick)"
    :class="highlighted ? 'bg-surface-raised' : 'hover:bg-surface-raised'"
    :aria-pressed="attached"
    :title="pickTitle"
    :data-walk-index="walkIndex"
    :data-attached="attached"
    data-testid="note-placement-row"
    @mousemove="emit('hover')"
    @click="emit('toggle')"
  >
    <span
      class="grid size-4 shrink-0 place-items-center rounded-key border transition-colors"
      :class="
        attached
          ? 'border-accent-deep bg-accent-soft text-accent'
          : 'border-border-strong text-transparent group-hover/row:border-accent/60'
      "
      aria-hidden="true"
    >
      <svg class="size-3" viewBox="0 0 24 24" fill="none">
        <path d="M5 12.5l4.5 4.5L19 7.5" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </span>

    <span class="min-w-0 flex-1">
      <span class="block truncate text-small leading-[1.35]" :class="attached ? 'text-text' : 'text-text-muted'">
        {{ task.title }}
      </span>
      <span class="mt-0.5 flex items-center gap-1.5">
        <span
          class="size-[6px] shrink-0 rounded-full"
          :class="TASK_STATUS_COLOR[task.status]"
          :title="TASK_STATUS_LABEL[task.status]"
        />
        <span class="anchor-chip truncate px-1.5 py-px text-micro leading-[15px]">{{ task.label }}</span>
      </span>
    </span>
  </button>
</template>
