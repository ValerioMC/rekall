<script setup lang="ts">
import { TASK_STATUS_COLOR } from '@/model/catalog'
import type { Task } from '@/model/catalog'

defineProps<{
  task: Task
  selected: boolean
  running: boolean
  showContext: boolean
  showCompany: boolean
}>()

defineEmits<{ select: []; edit: [] }>()
</script>

<template>
  <div class="group/task relative flex items-center">
    <button
      data-testid="task-row"
      class="focus-ring flex w-full items-center gap-2.5 rounded-[var(--radius-control)] px-2 py-2 text-left transition-colors"
      :class="selected ? 'selected-row text-text' : 'text-text-muted hover:bg-surface-raised hover:text-text'"
      :aria-current="selected"
      @click="$emit('select')"
    >
      <span class="size-[6px] shrink-0 rounded-full" :class="TASK_STATUS_COLOR[task.status]" aria-hidden="true" />

      <span class="min-w-0 flex-1">
        <span class="flex items-center gap-1.5 truncate text-[13px] font-medium">
          <span v-if="running" class="relative flex size-1.5 shrink-0" title="Tracking time on this task">
            <span class="absolute inline-flex size-full animate-ping rounded-full bg-accent opacity-75" />
            <span class="relative inline-flex size-1.5 rounded-full bg-accent" />
          </span>
          <span class="truncate">{{ task.title }}</span>
        </span>
        <span class="mt-0.5 flex items-center gap-1 truncate">
          <template v-if="showContext">
            <span v-if="showCompany" class="shrink-0 truncate text-[10px] text-text-subtle">
              {{ task.companyName }}
            </span>
            <span v-if="showCompany" class="shrink-0 text-[10px] text-text-subtle" aria-hidden="true">/</span>
          </template>
          <span class="anchor-chip shrink-0 truncate px-1.5 py-px text-[9.5px] leading-[15px]">
            {{ task.label }}
          </span>
        </span>
      </span>

      <span
        class="flex shrink-0 items-center gap-1.5 font-mono text-[10.5px] text-text-subtle transition-opacity group-hover/task:opacity-0"
      >
        <svg v-if="task.hasWrapup" class="size-2.5 text-text-muted" viewBox="0 0 12 12" role="img">
          <title>Has a wrapup</title>
          <path d="M6 1.2 10.8 6 6 10.8 1.2 6z" fill="currentColor" />
        </svg>
        {{ task.documentCount }}
      </span>
    </button>

    <button
      data-testid="edit-task"
      class="focus-ring absolute right-1.5 grid size-6 place-items-center rounded text-text-subtle opacity-0 transition hover:bg-surface-hover hover:text-accent focus-visible:opacity-100 group-hover/task:opacity-100"
      :aria-label="`Edit ${task.title}`"
      @click="$emit('edit')"
    >
      <svg class="size-3.5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
        <path
          d="M4 20h4L20 8l-4-4L4 16v4z"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linejoin="round"
        />
      </svg>
    </button>
  </div>
</template>
