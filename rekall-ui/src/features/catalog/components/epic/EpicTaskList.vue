<script setup lang="ts">
import { ref } from 'vue'
import AppCard from '@/components/ui/AppCard.vue'
import TagBadge from '@/components/ui/TagBadge.vue'
import TaskMark from '@/features/catalog/components/TaskMark.vue'
import EpicTaskPicker from '@/features/catalog/components/epic/EpicTaskPicker.vue'
import { TASK_STATUS_LABEL } from '@/features/catalog/model/catalog'
import type { Task, TaskStep } from '@/features/catalog/model/catalog'
import { taskMark } from '@/features/catalog/model/task-mark'
import type { TaskId } from '@/model/branded'

/** The tasks that belong to the epic, live work first. A row opens its task; its end takes it out of the epic. */
defineProps<{ epic: Task; children: readonly Task[]; steps: readonly TaskStep[]; runningTaskIds: ReadonlySet<TaskId> }>()

const emit = defineEmits<{ open: [id: TaskId]; attach: [id: TaskId]; create: []; remove: [task: Task] }>()

const isAttaching = ref(false)

function attach(id: TaskId): void {
  isAttaching.value = false
  emit('attach', id)
}
</script>

<template>
  <AppCard :padded="false" data-testid="epic-task-list">
    <header class="flex items-center gap-2 border-b border-border px-5 py-3">
      <span class="eyebrow text-caption">Tasks</span>
      <span class="rounded-full bg-surface px-1.5 py-px font-mono text-micro tabular-nums text-text-subtle">{{ children.length }}</span>
      <span class="ml-auto flex items-center gap-1.5">
        <button
          type="button"
          class="add-row focus-ring w-auto px-2.5 py-1"
          :class="isAttaching && 'add-row-open'"
          :aria-expanded="isAttaching"
          data-testid="epic-attach-task"
          @click="isAttaching = !isAttaching"
        >
          <svg class="size-3 shrink-0 text-accent" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M5 7l2-2M4.2 5.6L3 6.8a2 2 0 0 0 2.8 2.8L7 8.4M7.8 6.4L9 5.2A2 2 0 0 0 6.2 2.4L5 3.6" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
          </svg>
          Add existing task
        </button>
        <button type="button" class="add-row focus-ring w-auto px-2.5 py-1" data-testid="epic-create-task" @click="emit('create')">
          <span class="text-accent" aria-hidden="true">+</span> New task
        </button>
      </span>
    </header>

    <div v-if="isAttaching" class="border-b border-border px-5 py-3">
      <EpicTaskPicker :epic="epic" @pick="attach" @close="isAttaching = false" />
    </div>

    <p v-if="!children.length" class="px-5 py-6 text-small leading-relaxed text-text-subtle">
      No task belongs to this epic yet. Add one that exists, or create a new one here.
    </p>

    <ul v-else class="divide-y divide-border">
      <li v-for="task in children" :key="task.id" class="group/row flex items-center" :class="task.status === 'DONE' && 'filed-row'">
        <button
          class="focus-ring flex min-w-0 flex-1 items-center gap-3 py-2.5 pl-5 pr-2 text-left transition-colors hover:bg-surface-hover"
          data-testid="epic-task-row"
          @click="emit('open', task.id)"
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
        <button
          type="button"
          class="focus-ring mr-3 grid size-7 shrink-0 place-items-center rounded-control text-text-subtle opacity-0 transition-[opacity,color,background-color] duration-(--motion-quick) hover:bg-danger-soft hover:text-danger focus-visible:opacity-100 group-hover/row:opacity-100 group-focus-within/row:opacity-100"
          :title="`Take ${task.title} out of the epic`"
          :aria-label="`Take ${task.title} out of the epic`"
          data-testid="epic-remove-task"
          @click="emit('remove', task)"
        >
          <svg class="size-3" viewBox="0 0 12 12" fill="none" aria-hidden="true">
            <path d="M3 3l6 6M9 3l-6 6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
      </li>
    </ul>
  </AppCard>
</template>
