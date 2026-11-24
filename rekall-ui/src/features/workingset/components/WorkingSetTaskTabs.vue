<script setup lang="ts">
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import type { Project, Task } from '@/features/catalog/model/catalog'
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL } from '@/features/catalog/model/task-status'
import type { OpenPlace } from '@/features/workingset/model/working-set'
import type { TaskId } from '@/model/branded'

/**
 * Level 2 of the working-set bar, for the project in view: the project's task list first, then a
 * tab per open task. A tab's dot is the task's own status. The cross closes a task and selects
 * nothing; the plus goes to the task list, where another task is opened.
 */
const props = defineProps<{
  project: Project
  tasks: readonly Task[]
  currentTaskId: TaskId | null
  reviewCounts: ReadonlyMap<TaskId, number>
}>()

const emit = defineEmits<{ select: [place: OpenPlace]; close: [taskId: TaskId] }>()

function selectTask(taskId: TaskId): void {
  emit('select', { projectId: props.project.id, taskId })
}

function selectList(): void {
  emit('select', { projectId: props.project.id, taskId: null })
}
</script>

<template>
  <div
    class="flex h-[38px] shrink-0 items-stretch gap-0.5 overflow-x-auto overflow-y-hidden border-b border-border bg-canvas px-3"
    role="tablist"
    :aria-label="`${project.title} tasks`"
    data-testid="working-set-tasks"
  >
    <button
      type="button"
      role="tab"
      class="focus-ring relative flex shrink-0 cursor-pointer items-center gap-2 whitespace-nowrap pl-2.5 pr-3.5 font-medium transition-colors duration-(--motion-quick) hover:bg-surface hover:text-text"
      :class="currentTaskId === null ? 'text-text after:absolute after:inset-x-0 after:-bottom-px after:h-0.5 after:rounded-full after:bg-accent' : 'text-text-muted'"
      :aria-selected="currentTaskId === null"
      data-testid="working-set-all-tasks"
      @click="selectList"
    >
      All tasks
      <span class="text-caption tabular-nums text-text-subtle">{{ project.taskCount }}</span>
    </button>
    <span class="my-2.5 mx-1.5 w-px shrink-0 bg-border" aria-hidden="true"></span>
    <div
      v-for="task in tasks"
      :key="task.id"
      class="group/tab relative flex max-w-[250px] shrink-0"
      data-testid="working-set-task-tab"
    >
      <button
        type="button"
        role="tab"
        class="focus-ring relative flex min-w-0 cursor-pointer items-center gap-2 whitespace-nowrap pl-3 pr-7 transition-colors duration-(--motion-quick) hover:bg-surface hover:text-text"
        :class="currentTaskId === task.id ? 'text-text after:absolute after:inset-x-0 after:-bottom-px after:h-0.5 after:rounded-full after:bg-accent' : 'text-text-muted'"
        :aria-selected="currentTaskId === task.id"
        :title="`${task.code} ${task.title}`"
        data-testid="working-set-task"
        @click="selectTask(task.id)"
      >
        <span
          class="size-2 shrink-0 rounded-full"
          :class="TASK_STATUS_COLOR[task.status]"
          role="img"
          :aria-label="TASK_STATUS_LABEL[task.status]"
        ></span>
        <span class="shrink-0 font-mono text-caption text-text-subtle">{{ task.code }}</span>
        <span class="truncate">{{ task.title }}</span>
        <span
          v-if="(reviewCounts.get(task.id) ?? 0) > 0"
          class="shrink-0 rounded-full bg-accent px-1.5 text-caption font-medium leading-4 text-accent-ink"
          data-testid="working-set-task-review"
        >
          {{ reviewCounts.get(task.id) }}
        </span>
      </button>
      <button
        type="button"
        class="focus-ring absolute right-1.5 top-1/2 grid size-[18px] -translate-y-1/2 cursor-pointer place-items-center rounded-key text-text-subtle opacity-0 transition-opacity duration-(--motion-quick) hover:bg-surface-hover hover:text-text focus-visible:opacity-100 group-hover/tab:opacity-100"
        :class="currentTaskId === task.id && 'opacity-100'"
        :aria-label="`Close ${task.code}`"
        :title="`Close ${task.code}`"
        data-testid="working-set-task-close"
        @click="emit('close', task.id)"
      >
        <CloseGlyph small />
      </button>
    </div>
    <button
      type="button"
      class="focus-ring ml-1 grid size-[26px] shrink-0 cursor-pointer place-items-center self-center rounded-key text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text"
      aria-label="Open another task from the task list"
      title="Open another task from the task list"
      data-testid="working-set-add"
      @click="selectList"
    >
      <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
        <path d="M7 2.5v9M2.5 7h9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
    </button>
  </div>
</template>
