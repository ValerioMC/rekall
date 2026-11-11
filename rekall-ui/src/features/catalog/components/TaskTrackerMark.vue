<script setup lang="ts">
import AppBadge from '@/components/ui/AppBadge.vue'
import type { Task } from '@/features/catalog/model/catalog'

/**
 * The issue the project's task manager holds for the task, as a link to it, and a warning with
 * the reason when the last write to it failed.
 */
defineProps<{ task: Pick<Task, 'trackerIssueKey' | 'trackerIssueUrl' | 'trackerSyncError'> }>()
</script>

<template>
  <span v-if="task.trackerIssueKey || task.trackerSyncError" class="inline-flex shrink-0 items-center gap-1">
    <a
      v-if="task.trackerIssueKey && task.trackerIssueUrl"
      :href="task.trackerIssueUrl"
      target="_blank"
      rel="noopener noreferrer"
      class="focus-ring rounded-control"
      :aria-label="`Open issue ${task.trackerIssueKey} in the task manager`"
      data-testid="tracker-issue-link"
    >
      <AppBadge tone="accent" mono>{{ task.trackerIssueKey }} &#8599;</AppBadge>
    </a>
    <AppBadge v-if="task.trackerSyncError" tone="warn" :title="task.trackerSyncError" data-testid="tracker-sync-error">
      Not synced
    </AppBadge>
  </span>
</template>
