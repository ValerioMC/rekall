<script setup lang="ts">
import type { TaskId } from '@/model/branded'

/** The title row of the full-size chat. The title leads to the task's page; with no task it leads nowhere. */
defineProps<{ title: string; taskId: TaskId | null }>()
</script>

<template>
  <header class="flex shrink-0 items-start gap-4 border-b border-border px-5 py-3" data-testid="chat-header">
    <div class="min-w-0 flex-1">
      <div class="flex min-w-0 items-center gap-2.5">
        <h2 class="min-w-0 text-heading font-semibold tracking-[-0.015em] text-text">
          <RouterLink
            v-if="taskId"
            :to="{ name: 'task', params: { id: taskId, tab: 'description' } }"
            class="focus-ring block truncate rounded-control hover:text-accent hover:underline hover:decoration-1 hover:underline-offset-4"
            data-testid="chat-task-link"
          >
            {{ title }}
          </RouterLink>
          <span v-else class="block truncate">{{ title }}</span>
        </h2>
        <slot name="status" />
      </div>
      <p class="mt-0.5 flex min-w-0 items-center gap-2 whitespace-nowrap text-caption text-text-muted">
        <slot name="detail" />
      </p>
    </div>
    <div class="flex shrink-0 items-center gap-2 pt-0.5">
      <slot name="actions" />
    </div>
  </header>
</template>
