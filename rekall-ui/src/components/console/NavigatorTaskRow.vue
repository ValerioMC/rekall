<script setup lang="ts">
import { TASK_STATUS_COLOR, TASK_STATUS_RING } from '@/model/catalog'
import type { Task } from '@/model/catalog'

withDefaults(
  defineProps<{
    task: Task
    selected: boolean
    running: boolean
    showContext: boolean
    showCompany: boolean
    /** In the filing drawer: the row sits at reduced weight and lifts on hover. */
    filed?: boolean
  }>(),
  { filed: false }
)

defineEmits<{ select: []; edit: [] }>()
</script>

<template>
  <div class="group/task relative" :class="filed && 'filed-row'">
    <button
      data-testid="task-row"
      class="focus-ring flex w-full items-start gap-2.5 rounded-[var(--radius-control)] px-2.5 py-2 text-left transition-colors"
      :class="selected ? 'selected-row text-text' : 'text-text-muted hover:bg-surface-raised hover:text-text'"
      :aria-current="selected"
      @click="$emit('select')"
    >
      <!--
        The status pip: a coloured core inside a soft halo of the same hue, so the state carries
        its own weight instead of being a single hard dot pushed against the title. While a timer
        runs it becomes a sweeping ping, because "worked right now" should read before the colour.
      -->
      <span class="relative mt-[3px] grid size-3 shrink-0 place-items-center" aria-hidden="true">
        <span
          class="absolute size-3 rounded-full"
          :class="running ? 'bg-accent/20' : TASK_STATUS_RING[task.status]"
        />
        <template v-if="running">
          <span class="absolute inline-flex size-3 animate-ping rounded-full bg-accent/55" />
          <span class="relative inline-flex size-2 rounded-full bg-accent" />
        </template>
        <span
          v-else
          class="relative rounded-full transition-all"
          :class="[TASK_STATUS_COLOR[task.status], selected ? 'size-2' : 'size-[7px]']"
        />
      </span>

      <span class="min-w-0 flex-1">
        <span class="block truncate text-[13px] font-medium leading-[1.35]">{{ task.title }}</span>
        <span class="mt-1 flex items-center gap-1.5 truncate">
          <template v-if="showContext && showCompany">
            <span class="shrink-0 truncate text-[10px] text-text-subtle">{{ task.companyName }}</span>
            <span class="shrink-0 text-[10px] text-text-subtle" aria-hidden="true">/</span>
          </template>
          <span class="anchor-chip shrink-0 truncate px-1.5 py-px text-[9.5px] leading-[15px]">
            {{ task.label }}
          </span>
        </span>
      </span>

      <!--
        Progress and shape of the task, kept in a steady column rather than swapped out on hover:
        a number that vanishes the moment you reach for it is a number you stop trusting.
      -->
      <span
        class="mt-[3px] flex shrink-0 items-center gap-1.5 font-mono text-[10.5px] text-text-subtle"
      >
        <span
          v-if="task.stepCount > 0"
          class="tabular-nums"
          :class="task.stepsDone === task.stepCount ? 'text-safe' : ''"
          :title="`${task.stepsDone} of ${task.stepCount} steps done`"
          data-testid="task-steps"
        >
          {{ task.stepsDone }}/{{ task.stepCount }}
        </span>
        <svg v-if="task.hasWrapup" class="size-2.5 text-text-muted" viewBox="0 0 12 12" role="img">
          <title>Has a wrapup</title>
          <path d="M6 1.2 10.8 6 6 10.8 1.2 6z" fill="currentColor" />
        </svg>
        <span class="tabular-nums">{{ task.documentCount }}</span>
      </span>
    </button>

    <!-- Edit rides in from the right on hover as its own chip, so it never has to displace the
         row's own content to make room. -->
    <button
      data-testid="edit-task"
      class="focus-ring absolute right-1.5 top-1/2 grid size-6 -translate-y-1/2 translate-x-1 place-items-center rounded-md border border-border-strong bg-surface-hover text-text-subtle opacity-0 shadow-lift transition-all hover:text-accent focus-visible:translate-x-0 focus-visible:opacity-100 group-hover/task:translate-x-0 group-hover/task:opacity-100"
      :aria-label="`Edit ${task.title}`"
      @click.stop="$emit('edit')"
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
