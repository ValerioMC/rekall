<script setup lang="ts">
import AppCard from '@/components/ui/AppCard.vue'
import TagBadge from '@/components/ui/TagBadge.vue'
import TaskMark from '@/features/catalog/components/TaskMark.vue'
import { TASK_STATUS_LABEL } from '@/features/catalog/model/catalog'
import type { Task, TaskStep } from '@/features/catalog/model/catalog'
import { taskMark } from '@/features/catalog/model/task-mark'
import type { TaskId } from '@/model/branded'

/** The tasks that belong to the epic, live work first; a row opens its task in the console. */
defineProps<{ children: readonly Task[]; steps: readonly TaskStep[]; runningTaskIds: ReadonlySet<TaskId> }>()

defineEmits<{ open: [id: TaskId]; add: [] }>()
</script>

<template>
  <AppCard :padded="false" data-testid="epic-task-list">
    <header class="flex items-center gap-2 border-b border-border px-5 py-3">
      <span class="eyebrow text-caption">Tasks</span>
      <span class="rounded-full bg-surface px-1.5 py-px font-mono text-micro tabular-nums text-text-subtle">{{ children.length }}</span>
      <button
        class="add-row focus-ring ml-auto w-auto px-2.5 py-1"
        data-testid="epic-add-task"
        @click="$emit('add')"
      >
        <span class="text-accent">+</span> Add a task
      </button>
    </header>

    <p v-if="!children.length" class="px-5 py-6 text-small leading-relaxed text-text-subtle">
      No task belongs to this epic yet. Add one here, or pick this epic in a task's Epic field.
    </p>

    <ul v-else class="divide-y divide-border">
      <li v-for="task in children" :key="task.id">
        <button
          class="focus-ring flex w-full items-center gap-3 px-5 py-2.5 text-left transition-colors hover:bg-surface-hover"
          :class="task.status === 'DONE' && 'filed-row'"
          data-testid="epic-task-row"
          @click="$emit('open', task.id)"
        >
          <span class="relative grid size-3.5 shrink-0 place-items-center">
            <TaskMark
              :state="taskMark(task, steps, runningTaskIds.has(task.id)).state"
              :status="task.status"
              :accepted="taskMark(task, steps, runningTaskIds.has(task.id)).accepted"
              :selected="false"
            />
          </span>
          <span class="min-w-0 flex-1">
            <span class="flex items-center gap-2">
              <span class="min-w-0 truncate text-body font-medium text-text">{{ task.title }}</span>
              <TagBadge
                v-if="task.tagId && task.tagName && task.tagIcon && task.tagColor"
                class="shrink-0"
                :name="task.tagName"
                :icon="task.tagIcon"
                :color="task.tagColor"
              />
            </span>
            <span class="anchor-chip mt-0.5 inline-block max-w-full truncate px-1.5 py-px text-micro leading-[15px]">{{ task.label }}</span>
          </span>
          <span class="hidden w-28 shrink-0 sm:block" :title="`${task.stepsDone} of ${task.stepCount} steps done`">
            <template v-if="task.stepCount > 0">
              <span class="trough flex h-[6px] p-[1px]">
                <span
                  class="tube-gold h-full rounded-full"
                  :class="task.stepsDone === task.stepCount && 'tube-safe'"
                  :style="{ width: `${(task.stepsDone / task.stepCount) * 100}%` }"
                />
              </span>
              <span class="mt-1 block text-right font-mono text-micro tabular-nums text-text-subtle">
                {{ task.stepsDone }}/{{ task.stepCount }} steps
              </span>
            </template>
            <span v-else class="block text-right text-micro text-text-subtle">No checklist</span>
          </span>
          <span class="w-24 shrink-0 text-right text-caption text-text-muted">{{ TASK_STATUS_LABEL[task.status] }}</span>
        </button>
      </li>
    </ul>
  </AppCard>
</template>
