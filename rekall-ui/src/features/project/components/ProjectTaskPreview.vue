<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import TagBadge from '@/components/ui/TagBadge.vue'
import ProjectTaskPeek from '@/features/project/components/ProjectTaskPeek.vue'
import TaskStatusMenu from '@/features/task/components/TaskStatusMenu.vue'
import { formatDuration } from '@/common/format/duration'
import { useProjectTasksStore } from '@/features/project/store/projectTasks.store'
import { useConsoleStore } from '@/stores/console.store'
import type { ListedTask } from '@/features/project/model/project-task-list'

/** The pane beside a wide task list: the picked task's facts and everything its row would open to. */
const props = defineProps<{ row: ListedTask }>()

const store = useConsoleStore()
const projectTasks = useProjectTasksStore()

const task = computed(() => props.row.task)
const epicTitle = computed(() => store.tasks.find((candidate) => candidate.id === task.value.epicId)?.title ?? null)
const plan = computed(() =>
  task.value.stepCount > 0 ? `${task.value.stepsDone} of ${task.value.stepCount} steps done` : 'No steps'
)
</script>

<template>
  <aside class="w-(--spacing-inspector) shrink-0 border-l border-border" aria-label="Task preview" data-testid="task-preview">
    <div class="sticky top-0 max-h-screen overflow-y-auto py-3.5 pl-4">
      <TaskStatusMenu :row="row" @changed="projectTasks.linger(task.id)" />
      <h3 class="mt-2 text-title font-semibold text-text">
        <RouterLink
          :to="`/task/${task.id}/description`"
          class="focus-ring rounded-key decoration-accent underline-offset-4 hover:underline"
          data-testid="task-preview-title"
        >
          {{ task.title }}
        </RouterLink>
      </h3>
      <dl class="my-3 grid grid-cols-[auto_minmax(0,1fr)] gap-x-4 gap-y-1.5 text-small text-text">
        <dt class="text-text-subtle">Tag</dt>
        <dd class="flex min-w-0">
          <TagBadge v-if="task.tagName && task.tagIcon && task.tagColor" :name="task.tagName" :icon="task.tagIcon" :color="task.tagColor" />
          <span v-else class="text-text-muted">None</span>
        </dd>
        <dt class="text-text-subtle">Plan</dt>
        <dd>{{ plan }}</dd>
        <dt class="text-text-subtle">Epic</dt>
        <dd class="truncate">{{ epicTitle ?? 'None' }}</dd>
        <dt class="text-text-subtle">Time</dt>
        <dd>{{ row.trackedSeconds > 0 ? formatDuration(row.trackedSeconds) : 'None tracked' }}</dd>
      </dl>
      <ProjectTaskPeek :key="task.id" :row="row" />
    </div>
  </aside>
</template>
