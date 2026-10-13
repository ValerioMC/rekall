<script setup lang="ts">
import CopyAnchorButton from '@/components/console/CopyAnchorButton.vue'
import TaskTokenChip from '@/components/console/TaskTokenChip.vue'
import type { Task } from '@/model/catalog'

/**
 * The task's name with what used to take a second line beside it: its status (`flags`), a button
 * that copies its `/rk` command, its token count and trend (unless `showTokens` is off, for a
 * header that shows them elsewhere), and any `trailing` chip.
 */
withDefaults(defineProps<{ task: Task; copyTestid: string; showTokens?: boolean }>(), { showTokens: true })
</script>

<template>
  <div class="flex min-w-0 items-center gap-2">
    <h2 class="min-w-0 truncate text-heading font-semibold tracking-[-0.015em] text-text">
      {{ task.title }}
    </h2>
    <slot name="flags" />
    <CopyAnchorButton :anchor="task.anchor" :data-testid="copyTestid" />
    <TaskTokenChip v-if="showTokens" :task="task" />
    <slot name="trailing" />
  </div>
</template>
