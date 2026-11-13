<script setup lang="ts">
import { RouterLink } from 'vue-router'
import TaskActions from '@/features/task/components/TaskActions.vue'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * The two lines above the tabs: where the task sits with what to do to it on the same line, and
 * its title alone beneath. Status, anchor and width live on the tab strip.
 */
defineProps<{ task: Task }>()
</script>

<template>
  <header class="px-6 pt-2.5" data-testid="task-header">
    <div class="flex min-h-8 items-center gap-3 @max-[46rem]/task:[&_[data-label]]:hidden">
      <nav class="flex min-w-0 flex-1 items-center gap-2 text-small text-text-muted" aria-label="Where this task is">
        <RouterLink :to="{ name: 'work' }" class="focus-ring shrink-0 rounded-key hover:text-text" data-testid="crumb-work">Work</RouterLink>
        <span class="text-text-subtle" aria-hidden="true">/</span>
        <RouterLink
          :to="{ name: 'project-detail', params: { id: task.projectId } }"
          class="focus-ring min-w-0 truncate rounded-key hover:text-text"
          data-testid="crumb-project"
        >
          {{ task.companyName }} · {{ task.projectTitle }}
        </RouterLink>
      </nav>
      <TaskActions :task="task" />
    </div>

    <h1 class="mt-0.5 truncate text-heading font-semibold tracking-[-0.015em] text-text" data-testid="task-title" :title="task.title">
      {{ task.title }}
    </h1>
  </header>
</template>
