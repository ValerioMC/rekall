<script setup lang="ts">
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'
import QuickTaskMark from '@/features/catalog/components/QuickTaskMark.vue'
import TaskTokenChip from '@/features/tokens/components/TaskTokenChip.vue'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * The task's name with what used to take a second line beside it: its status (`flags`), a button
 * that copies its `/rk` command, its generated tokens with their trend, and any `trailing` chip
 * to the right of them.
 */
defineProps<{ task: Task; copyTestid: string }>()
</script>

<template>
  <div class="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1">
    <h2 class="min-w-[12ch] max-w-full line-clamp-2 text-heading font-semibold tracking-[-0.015em] text-text">
      {{ task.title }}
    </h2>
    <QuickTaskMark :task="task" />
    <slot name="flags" />
    <CopyAnchorButton :anchor="task.anchor" :data-testid="copyTestid" />
    <TaskTokenChip :task="task" />
    <slot name="trailing" />
  </div>
</template>
