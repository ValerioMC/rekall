<script setup lang="ts">
import { computed } from 'vue'
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL } from '@/model/catalog'
import type { Task } from '@/model/catalog'

/**
 * One task in a note-side picker: a tick that says whether the note is (or will be) on it, the
 * task's title, status and label. Shared by the placements column and the new-note composer, so
 * putting a note on a task looks the same whether the note exists yet or not.
 * `locked` is the placements column's "only here": the row is shown but cannot be flipped.
 */
const props = withDefaults(
  defineProps<{
    task: Task
    attached: boolean
    walkIndex: number
    highlighted?: boolean
    busy?: boolean
    locked?: boolean
    openable?: boolean
  }>(),
  { highlighted: false, busy: false, locked: false, openable: false }
)

const emit = defineEmits<{ toggle: []; open: []; hover: [] }>()

const title = computed(() => {
  if (props.locked) return 'The only task this note is on. A note needs at least one.'
  return props.attached
    ? `Take this note off ${props.task.title}`
    : `Put this note on ${props.task.title}`
})
</script>

<template>
  <div class="group/row relative">
    <button
      type="button"
      class="focus-ring flex w-full items-center gap-2.5 rounded-[var(--radius-control)] py-1.5 pl-2 text-left transition-colors"
      :class="[
        openable && attached ? 'pr-8' : 'pr-2',
        highlighted ? 'bg-surface-raised' : 'hover:bg-surface-raised',
        locked ? 'cursor-default' : 'cursor-pointer',
        busy && 'opacity-60'
      ]"
      :aria-pressed="attached"
      :aria-disabled="locked"
      :title="title"
      :data-walk-index="walkIndex"
      :data-attached="attached"
      data-testid="note-placement-row"
      @mousemove="emit('hover')"
      @click="emit('toggle')"
    >
      <span
        class="grid size-4 shrink-0 place-items-center rounded-[5px] border transition-colors"
        :class="
          attached
            ? 'border-accent-deep bg-accent-soft text-accent'
            : 'border-border-strong text-transparent group-hover/row:border-accent/60'
        "
        aria-hidden="true"
      >
        <svg class="size-3" viewBox="0 0 24 24" fill="none">
          <path
            d="M5 12.5l4.5 4.5L19 7.5"
            stroke="currentColor"
            stroke-width="2.6"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
      </span>

      <span class="min-w-0 flex-1">
        <span
          class="block truncate text-[12.5px] leading-[1.35]"
          :class="attached ? 'text-text' : 'text-text-muted'"
        >
          {{ task.title }}
        </span>
        <span class="mt-0.5 flex items-center gap-1.5">
          <span
            class="size-[6px] shrink-0 rounded-full"
            :class="TASK_STATUS_COLOR[task.status]"
            :title="TASK_STATUS_LABEL[task.status]"
          />
          <span class="anchor-chip truncate px-1.5 py-px text-[9.5px] leading-[15px]">
            {{ task.label }}
          </span>
          <span v-if="locked" class="truncate text-[10px] text-accent/80">only here</span>
        </span>
      </span>
    </button>

    <button
      v-if="openable && attached"
      type="button"
      class="focus-ring absolute right-1.5 top-1/2 grid size-6 -translate-y-1/2 place-items-center rounded-md text-text-subtle opacity-0 transition-opacity hover:bg-surface-hover hover:text-accent focus-visible:opacity-100 group-hover/row:opacity-100"
      :title="`Open ${task.title} on the Tasks side`"
      :aria-label="`Open ${task.title}`"
      data-testid="note-placement-open"
      @click.stop="emit('open')"
    >
      <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
        <path
          d="M2.5 6h7M6.5 3l3 3-3 3"
          stroke="currentColor"
          stroke-width="1.4"
          stroke-linecap="round"
          stroke-linejoin="round"
        />
      </svg>
    </button>
  </div>
</template>
